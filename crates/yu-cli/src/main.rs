use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{ExitCode, Termination},
};
use yu_capability_image::{ImageEngine, ImageErrorKind, ImageOperationError, ResizeRequest};
use yu_core::{
    EngineDescriptor, EngineProvider, EngineState, ErrorCode, ErrorEnvelope, ResultEnvelope,
    RuntimeRegistry, YuError,
};
use yu_engine_image_rs::{ENGINE_ID as RASTER_ENGINE_ID, RustImageEngine};
use yu_engine_manager::{
    EngineInstaller, EngineManager, EngineManifest, HttpDownloader, ManagerError,
};

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
    /// List known built-in engines and their current state.
    List,
    /// Install a managed engine from a local manifest.
    Install {
        #[arg(long)]
        manifest: PathBuf,
    },
    /// List installed versions of a managed engine.
    Versions {
        engine: String,
    },
    /// Activate an installed managed-engine version.
    Activate {
        engine: String,
        version: String,
    },
    /// Deactivate a managed engine without removing installed versions.
    Deactivate {
        engine: String,
    },
    /// Remove an inactive managed-engine version.
    Remove {
        engine: String,
        version: String,
    },
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
        Command::Engine {
            command: EngineCommand::Install { manifest },
        } => render_engine_install(&manifest, cli.json),
        Command::Engine {
            command: EngineCommand::Versions { engine },
        } => render_engine_versions(&engine, cli.json),
        Command::Engine {
            command: EngineCommand::Activate { engine, version },
        } => render_engine_activate(&engine, &version, cli.json),
        Command::Engine {
            command: EngineCommand::Deactivate { engine },
        } => render_engine_deactivate(&engine, cli.json),
        Command::Engine {
            command: EngineCommand::Remove { engine, version },
        } => render_engine_remove(&engine, &version, cli.json),
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


fn render_engine_install(manifest_path: &Path, json: bool) -> Result<(), YuError> {
    let manifest = read_manifest(manifest_path)?;
    let manager = EngineManager::discover().map_err(map_manager_error)?;
    let downloader = HttpDownloader::new().map_err(map_manager_error)?;
    let installer = EngineInstaller::new(manager, downloader);
    let receipt = installer.install(&manifest).map_err(map_manager_error)?;

    if json {
        print_json(&ResultEnvelope::new("engine.install", receipt));
    } else {
        println!("Installed {} {}", receipt.engine_id, receipt.version);
        println!("Target: {}/{}", receipt.target_os, receipt.target_arch);
        println!("Entrypoint: {}", receipt.entrypoint.display());
        println!("Active version: unchanged");
    }
    Ok(())
}

fn render_engine_versions(engine_id: &str, json: bool) -> Result<(), YuError> {
    let manager = EngineManager::discover().map_err(map_manager_error)?;
    let descriptor = managed_descriptor(engine_id);
    let versions = manager
        .list_managed_versions(&descriptor)
        .map_err(map_manager_error)?;

    if json {
        print_json(&ResultEnvelope::new("engine.versions", versions));
    } else if versions.is_empty() {
        println!("No managed versions installed for {engine_id}");
    } else {
        println!("{:<16} {:<8} PATH", "VERSION", "ACTIVE");
        for version in versions {
            println!(
                "{:<16} {:<8} {}",
                version.version,
                if version.active { "yes" } else { "no" },
                version.path.display()
            );
        }
    }
    Ok(())
}

fn render_engine_activate(engine_id: &str, version: &str, json: bool) -> Result<(), YuError> {
    let manager = EngineManager::discover().map_err(map_manager_error)?;
    let descriptor = managed_descriptor(engine_id);
    let receipt = manager
        .activate_version(&descriptor, version)
        .map_err(map_manager_error)?;

    if json {
        print_json(&ResultEnvelope::new("engine.activate", receipt));
    } else {
        println!(
            "Active {}: {}{}",
            receipt.engine_id,
            receipt.active_version,
            if receipt.changed { "" } else { " (unchanged)" }
        );
    }
    Ok(())
}

fn render_engine_deactivate(engine_id: &str, json: bool) -> Result<(), YuError> {
    let manager = EngineManager::discover().map_err(map_manager_error)?;
    let descriptor = managed_descriptor(engine_id);
    let receipt = manager
        .deactivate(&descriptor)
        .map_err(map_manager_error)?;

    if json {
        print_json(&ResultEnvelope::new("engine.deactivate", receipt));
    } else if receipt.changed {
        println!(
            "Deactivated {} (previous: {})",
            receipt.engine_id,
            receipt.previous_version.as_deref().unwrap_or("-")
        );
    } else {
        println!("{} is already inactive", receipt.engine_id);
    }
    Ok(())
}

fn render_engine_remove(engine_id: &str, version: &str, json: bool) -> Result<(), YuError> {
    let manager = EngineManager::discover().map_err(map_manager_error)?;
    let descriptor = managed_descriptor(engine_id);
    let receipt = manager
        .remove_version(&descriptor, version)
        .map_err(map_manager_error)?;

    if json {
        print_json(&ResultEnvelope::new("engine.remove", receipt));
    } else {
        println!("Removed {} {}", receipt.engine_id, receipt.version);
        if !receipt.cleanup_complete {
            println!(
                "Cleanup pending: {}",
                receipt
                    .cleanup_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "-".to_owned())
            );
        }
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<EngineManifest, YuError> {
    let bytes = fs::read(path).map_err(|error| {
        YuError::new(
            ErrorCode::InvalidInput,
            format!("cannot read engine manifest {}: {error}", path.display()),
        )
    })?;

    let manifest: EngineManifest = serde_json::from_slice(&bytes).map_err(|error| {
        YuError::new(
            ErrorCode::InvalidInput,
            format!("cannot parse engine manifest {}: {error}", path.display()),
        )
    })?;
    manifest.validate().map_err(map_manager_error)?;
    Ok(manifest)
}

fn managed_descriptor(engine_id: &str) -> EngineDescriptor {
    EngineDescriptor {
        id: engine_id.to_owned(),
        display_name: engine_id.to_owned(),
        provider: EngineProvider::Managed,
        state: EngineState::Ready,
        version: None,
        capabilities: Vec::new(),
    }
}

fn map_manager_error(error: ManagerError) -> YuError {
    let code = match error {
        ManagerError::InvalidManifest(_) | ManagerError::Archive(_) => ErrorCode::InvalidInput,
        ManagerError::Incompatible(_) | ManagerError::Ownership(_) => ErrorCode::EngineIncompatible,
        ManagerError::NotInstalled(_) => ErrorCode::EngineUnavailable,
        ManagerError::AlreadyInstalled(_)
        | ManagerError::ActiveVersion(_)
        | ManagerError::Busy(_) => ErrorCode::OutputConflict,
        ManagerError::Integrity(_) => ErrorCode::VerificationFailed,
        ManagerError::Download(_)
        | ManagerError::State(_)
        | ManagerError::Environment(_)
        | ManagerError::Io(_) => ErrorCode::ExecutionFailed,
    };

    YuError::new(code, error.to_string())
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
