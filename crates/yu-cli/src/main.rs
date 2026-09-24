use clap::{Parser, Subcommand};
use serde::Serialize;
use yu_core::{RuntimeRegistry, SCHEMA_VERSION};

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
}

#[derive(Debug, Subcommand)]
enum EngineCommand {
    /// List known engines and their current state.
    List,
}

#[derive(Debug, Serialize)]
struct Envelope<T> {
    schema_version: &'static str,
    operation: &'static str,
    result: T,
}

fn main() {
    let cli = Cli::parse();
    let registry = RuntimeRegistry::bootstrap();

    match cli.command {
        Command::Doctor => render_doctor(&registry, cli.json),
        Command::Capabilities => render_capabilities(&registry, cli.json),
        Command::Engine {
            command: EngineCommand::List,
        } => render_engines(&registry, cli.json),
    }
}

fn render_doctor(registry: &RuntimeRegistry, json: bool) {
    let report = registry.doctor_report();

    if json {
        print_json(&Envelope {
            schema_version: SCHEMA_VERSION,
            operation: "runtime.doctor",
            result: report,
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
            result: registry.capabilities(),
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
            result: registry.engines(),
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

fn print_json<T: Serialize>(value: &T) {
    let rendered =
        serde_json::to_string_pretty(value).expect("serializing YuTool output should not fail");
    println!("{rendered}");
}
