use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};
use yu_capability_image::{ImageEngine, ImageErrorKind, ImageOperationError, ResizeRequest};
use yu_core::{
    EngineDescriptor, ResolveError, ResolveErrorKind, RuntimeRegistry, SCHEMA_VERSION,
};
use yu_engine_image_rs::{ENGINE_ID as RASTER_ENGINE_ID, RustImageEngine};

#[derive(Debug, Parser)]
#[command(
    name = "yu",
    version,
    about = "Lightweight local tool runtime for developers and AI agents"
)]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON")]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Diagnose YuTool runtime and engine health.
    Doctor,
    /// List capabilities available on this machine.
    Capabilities,
    /// Inspect and manage engines.
    Engine {
        #[command(subcommand)]
        command: EngineCommand,
    },
    /// Inspect and transform raster images.
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
}

#[derive(Debug, Subcommand)]
enum EngineCommand {
    /// List known engines and their current state.
    List,
}

#[derive(Debug, Subcommand)]
enum ImageCommand {
    /// Inspect image dimensions, format, and color information.
    Info {
        file: PathBuf,

        #[arg(long, help = "Use a specific engine instead of automatic resolution")]
        engine: Option<String>,
    },
    /// Resize an image to a new output file.
    Resize {
        input: PathBuf,

        #[arg(long)]
        width: Option<u32>,

        #[arg(long)]
        height: Option<u32>,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(long, help = "Use a specific engine instead of automatic resolution")]
        engine: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct EngineRef<'a> {
    id: &'a str,
    provider: yu_core::EngineProvider,
    version: Option<&'a str>,
}

impl<'a> From<&'a EngineDescriptor> for EngineRef<'a> {
    fn from(engine: &'a EngineDescriptor) -> Self {
        Self {
            id: &engine.id,
            provider: engine.provider,
            version: engine.version.as_deref(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Envelope<'a, T> {
    schema_version: &'static str,
    operation: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    engine: Option<EngineRef<'a>>,
    result: T,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
    schema_version: &'static str,
    error: StructuredError<'a>,
}

#[derive(Debug, Serialize)]
struct StructuredError<'a> {
    code: &'static str,
    message: &'a str,
}

#[derive(Debug)]
struct AppError {
    code: &'static str,
    message: String,
    exit_code: u8,
}

fn main() -> impl Termination {
    let cli = Cli::parse();
    let json = cli.json;

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            render_error(&error, json);
            ExitCode::from(error.exit_code)
        }
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    let registry = RuntimeRegistry::bootstrap();

    match cli.command {
        Command::Doctor => {
            render_doctor(&registry, cli.json);
            Ok(())
        }
        Command::Capabilities => {
            render_capabilities(&registry, cli.json);
            Ok(())
        }
        Command::Engine {
            command: EngineCommand::List,
        } => {
            render_engines(&registry, cli.json);
            Ok(())
        }
        Command::Image {
            command: ImageCommand::Info { file, engine },
        } => render_image_info(&registry, file, engine.as_deref(), cli.json),
        Command::Image {
            command:
                ImageCommand::Resize {
                    input,
                    width,
                    height,
                    output,
                    engine,
                },
        } => render_image_resize(
            &registry,
            ResizeRequest {
                input,
                output,
                width,
                height,
            },
            engine.as_deref(),
            cli.json,
        ),
    }
}

fn render_doctor(registry: &RuntimeRegistry, json: bool) {
    let report = registry.doctor_report();

    if json {
        print_json(&Envelope {
            schema_version: SCHEMA_VERSION,
            operation: "runtime.doctor",
            engine: None,
            result: report,
            warnings: Vec::new(),
        });
        return;
    }

    println!("YuTool {}", report.version);
    println!(
        "Status: {}",
        if report.healthy {
            "healthy"
        } else {
            "degraded"
        }
    );
    println!("Platform: {}/{}", report.platform.os, report.platform.arch);
    println!("Capabilities: {}", report.capabilities);
    println!(
        "Engines: {} total, {} ready",
        report.engines.total, report.engines.ready
    );
}

fn render_capabilities(registry: &RuntimeRegistry, json: bool) {
    if json {
        print_json(&Envelope {
            schema_version: SCHEMA_VERSION,
            operation: "runtime.capabilities",
            engine: None,
            result: registry.capabilities(),
            warnings: Vec::new(),
        });
        return;
    }

    println!("{:<24} {:<16} DESCRIPTION", "CAPABILITY", "ENGINES");
    for capability in registry.capabilities() {
        println!(
            "{:<24} {:<16} {}",
            capability.id,
            capability.engines.join(","),
            capability.summary
        );
    }
}

fn render_engines(registry: &RuntimeRegistry, json: bool) {
    if json {
        print_json(&Envelope {
            schema_version: SCHEMA_VERSION,
            operation: "engine.list",
            engine: None,
            result: registry.engines(),
            warnings: Vec::new(),
        });
        return;
    }

    println!(
        "{:<16} {:<12} {:<14} {:<10} CAPABILITIES",
        "ENGINE", "PROVIDER", "STATE", "VERSION"
    );
    for engine in registry.engines() {
        println!(
            "{:<16} {:<12} {:<14} {:<10} {}",
            engine.id,
            engine.provider,
            engine.state,
            engine.version.as_deref().unwrap_or("-"),
            engine.capabilities.len()
        );
    }
}

fn render_image_info(
    registry: &RuntimeRegistry,
    file: PathBuf,
    requested_engine: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let descriptor = registry
        .resolve_engine("image.info", requested_engine)
        .map_err(AppError::from)?;
    let engine = image_engine(descriptor)?;
    let result = engine.info(&file).map_err(AppError::from)?;

    if json {
        print_json(&Envelope {
            schema_version: SCHEMA_VERSION,
            operation: "image.info",
            engine: Some(descriptor.into()),
            result,
            warnings: Vec::new(),
        });
        return Ok(());
    }

    println!("Path: {}", result.path);
    println!("Format: {}", result.format);
    println!("Size: {}x{}", result.width, result.height);
    println!("Color: {}", result.color_type);
    println!("Bit depth: {}", result.bit_depth);
    println!("Channels: {}", result.channels);
    println!("Alpha: {}", result.has_alpha);
    println!("Engine: {}", descriptor.id);
    Ok(())
}

fn render_image_resize(
    registry: &RuntimeRegistry,
    request: ResizeRequest,
    requested_engine: Option<&str>,
    json: bool,
) -> Result<(), AppError> {
    let descriptor = registry
        .resolve_engine("image.resize", requested_engine)
        .map_err(AppError::from)?;
    let engine = image_engine(descriptor)?;
    let result = engine.resize(&request).map_err(AppError::from)?;

    if json {
        print_json(&Envelope {
            schema_version: SCHEMA_VERSION,
            operation: "image.resize",
            engine: Some(descriptor.into()),
            result,
            warnings: Vec::new(),
        });
        return Ok(());
    }

    println!(
        "Resized {} -> {} ({}x{} -> {}x{})",
        result.input,
        result.output,
        result.source_width,
        result.source_height,
        result.width,
        result.height
    );
    println!("Format: {}", result.format);
    println!("Engine: {}", descriptor.id);
    Ok(())
}

fn image_engine(descriptor: &EngineDescriptor) -> Result<RustImageEngine, AppError> {
    if descriptor.id == RASTER_ENGINE_ID {
        return Ok(RustImageEngine);
    }

    Err(AppError {
        code: "ENGINE_UNAVAILABLE",
        message: format!("image engine {} is not wired into this build", descriptor.id),
        exit_code: 3,
    })
}

impl From<ResolveError> for AppError {
    fn from(error: ResolveError) -> Self {
        let code = match error.kind {
            ResolveErrorKind::UnknownEngine | ResolveErrorKind::EngineUnavailable => {
                "ENGINE_UNAVAILABLE"
            }
            ResolveErrorKind::EngineIncompatible => "ENGINE_INCOMPATIBLE",
            ResolveErrorKind::NoCompatibleEngine => "UNSUPPORTED_CAPABILITY",
        };

        Self {
            code,
            message: error.message,
            exit_code: 3,
        }
    }
}

impl From<ImageOperationError> for AppError {
    fn from(error: ImageOperationError) -> Self {
        let (code, exit_code) = match error.kind {
            ImageErrorKind::InvalidInput => ("INVALID_INPUT", 2),
            ImageErrorKind::Unsupported => ("UNSUPPORTED_CAPABILITY", 3),
            ImageErrorKind::OutputConflict => ("OUTPUT_CONFLICT", 2),
            ImageErrorKind::Execution => ("EXECUTION_FAILED", 1),
        };

        Self {
            code,
            message: error.message,
            exit_code,
        }
    }
}

fn render_error(error: &AppError, json: bool) {
    if json {
        let envelope = ErrorEnvelope {
            schema_version: SCHEMA_VERSION,
            error: StructuredError {
                code: error.code,
                message: &error.message,
            },
        };
        let rendered =
            serde_json::to_string_pretty(&envelope).expect("serializing YuTool error should not fail");
        eprintln!("{rendered}");
    } else {
        eprintln!("error [{}]: {}", error.code, error.message);
    }
}

fn print_json<T: Serialize>(value: &T) {
    let rendered =
        serde_json::to_string_pretty(value).expect("serializing YuTool output should not fail");
    println!("{rendered}");
}
