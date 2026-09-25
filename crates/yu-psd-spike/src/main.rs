use std::env;
use std::path::Path;
use std::process;
use yu_psd_spike::{
    candidate_adapters, load_candidate_comparison, load_corpus, run_candidate,
};

fn main() {
    if let Err(message) = run() {
        eprintln!("error: {message}");
        eprintln!("{}", usage());
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.as_slice() {
        [command, corpus] if command == "validate" => {
            let corpus_path = Path::new(corpus);
            let loaded = load_corpus(corpus_path).map_err(|error| error.to_string())?;
            println!(
                "PSD corpus schema {} is valid ({} fixture(s))",
                loaded.schema_version,
                loaded.fixtures.len()
            );
            Ok(())
        }
        [command] if command == "candidates" => {
            let descriptors = candidate_adapters()
                .into_iter()
                .map(|adapter| adapter.descriptor())
                .collect::<Vec<_>>();
            let json = serde_json::to_string_pretty(&descriptors)
                .map_err(|error| format!("failed to serialize candidates: {error}"))?;
            println!("{json}");
            Ok(())
        }
        [command, comparison] if command == "comparison" => {
            let snapshot =
                load_candidate_comparison(Path::new(comparison)).map_err(|error| error.to_string())?;
            let json = serde_json::to_string_pretty(&snapshot)
                .map_err(|error| format!("failed to serialize comparison: {error}"))?;
            println!("{json}");
            Ok(())
        }
        [command, candidate_id, corpus] if command == "run" => {
            let adapter = candidate_adapters()
                .into_iter()
                .find(|adapter| adapter.descriptor().id == *candidate_id)
                .ok_or_else(|| format!("unknown PSD candidate: {candidate_id}"))?;
            let report = run_candidate(Path::new(corpus), adapter.as_ref())
                .map_err(|error| error.to_string())?;
            let json = serde_json::to_string_pretty(&report)
                .map_err(|error| format!("failed to serialize report: {error}"))?;
            println!("{json}");
            Ok(())
        }
        _ => Err("invalid arguments".to_owned()),
    }
}

fn usage() -> &'static str {
    "usage:\n  yu-psd-spike validate <corpus.json>\n  yu-psd-spike candidates\n  yu-psd-spike comparison <comparison.json>\n  yu-psd-spike run <candidate-id> <corpus.json>"
}
