use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{
    path::PathBuf,
    process::{ExitCode, Termination},
};
use yu_capability_image::{ImageEngine, ImageErrorKind, ImageOperationError, ResizeRequest};
use yu_core::{
    EngineDescriptor, ErrorCode, ErrorEnvelope, ResultEnvelope, RuntimeRegistry, YuError,
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

fn main() -> impl Termination {
    let cli = Cli::parse();
    let json = cli.json;

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            render_error(&error, json);
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: Cli) -> Result<(), YuError> {
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
        print_json(&ResultEnvelope::new("runtime.doctor", report));
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
        print_json(&ResultEnvelope::new(
            "runtime.capabilities",
            registry.capabilities(),
        ));
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
        print_json(&ResultEnvelope::new("engine.list", registry.engines()));
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
) -> Result<(), YuError> {
    let descriptor = registry.resolve_engine("image.info", requested_engine)?;
    let engine = image_engine(descriptor)?;
    let result = engine.info(&file).map_err(map_image_error)?;

    if json {
        print_json(&ResultEnvelope::new("image.info", result).with_engine(descriptor));
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
) -> Result<(), YuError> {
    let descriptor = registry.resolve_engine("image.resize", requested_engine)?;
    let engine = image_engine(descriptor)?;
    let result = engine.resize(&request).map_err(map_image_error)?;

    if json {
        print_json(&ResultEnvelope::new("image.resize", result).with_engine(descriptor));
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

fn image_engine(descriptor: &EngineDescriptor) -> Result<RustImageEngine, YuError> {
    if descriptor.id == RASTER_ENGINE_ID {
        return Ok(RustImageEngine);
    }

    Err(YuError::new(
        ErrorCode::EngineUnavailable,
        format!(
            "image engine {} is not wired into this build",
            descriptor.id
        ),
    ))
}

fn map_image_error(error: ImageOperationError) -> YuError {
    let code = match error.kind {
        ImageErrorKind::InvalidInput => ErrorCode::InvalidInput,
        ImageErrorKind::Unsupported => ErrorCode::UnsupportedCapability,
        ImageErrorKind::OutputConflict => ErrorCode::OutputConflict,
        ImageErrorKind::Execution => ErrorCode::ExecutionFailed,
    };

    YuError::new(code, error.message)
}

fn render_error(error: &YuError, json: bool) {
    if json {
        print_json_to_stderr(&ErrorEnvelope::from(error));
    } else {
        eprintln!("error {error}");
    }
}

fn print_json<T: Serialize>(value: &T) {
    let rendered =
        serde_json::to_string_pretty(value).expect("serializing YuTool output should not fail");
    println!("{rendered}");
}

fn print_json_to_stderr<T: Serialize>(value: &T) {
    let rendered =
        serde_json::to_string_pretty(value).expect("serializing YuTool error should not fail");
    eprintln!("{rendered}");
}
