use image::{Rgba, RgbaImage};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yu"))
        .args(args)
        .output()
        .expect("yu should execute")
}

fn parse_stdout(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn temp_path(label: &str, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "yu-cli-{label}-{}-{nonce}.{extension}",
        std::process::id()
    ))
}

fn create_png(path: &Path, width: u32, height: u32) {
    let image = RgbaImage::from_pixel(width, height, Rgba([32, 64, 128, 255]));
    image.save(path).expect("fixture should save");
}

#[test]
fn doctor_json_is_machine_readable() {
    let output = run(&["doctor", "--json"]);
    let json = parse_stdout(&output);

    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "runtime.doctor");
    assert_eq!(json["result"]["healthy"], true);
    assert_eq!(json["result"]["engines"]["ready"], 2);
}

#[test]
fn capabilities_json_contains_runtime_and_image_capabilities() {
    let output = run(&["capabilities", "--json"]);
    let json = parse_stdout(&output);
    let capabilities = json["result"]
        .as_array()
        .expect("result should be an array");

    assert!(
        capabilities
            .iter()
            .any(|item| item["id"] == "runtime.doctor")
    );
    assert!(
        capabilities
            .iter()
            .any(|item| item["id"] == "runtime.capabilities")
    );
    assert!(capabilities.iter().any(|item| item["id"] == "engine.list"));
    assert!(capabilities.iter().any(|item| item["id"] == "image.info"));
    assert!(capabilities.iter().any(|item| item["id"] == "image.resize"));
}

#[test]
fn engine_list_json_contains_built_in_runtime_and_raster_engine() {
    let output = run(&["engine", "list", "--json"]);
    let json = parse_stdout(&output);
    let engines = json["result"]
        .as_array()
        .expect("result should be an array");

    assert_eq!(engines.len(), 2);
    assert!(
        engines
            .iter()
            .any(|engine| engine["id"] == "yu-runtime" && engine["state"] == "ready")
    );
    assert!(
        engines
            .iter()
            .any(|engine| engine["id"] == "raster-rs" && engine["provider"] == "built_in")
    );
}

#[test]
fn image_info_json_reports_fixture_metadata() {
    let input = temp_path("info-input", "png");
    create_png(&input, 4, 2);

    let output = run(&["image", "info", input.to_str().unwrap(), "--json"]);
    let json = parse_stdout(&output);

    assert_eq!(json["operation"], "image.info");
    assert_eq!(json["engine"]["id"], "raster-rs");
    assert_eq!(json["result"]["format"], "png");
    assert_eq!(json["result"]["width"], 4);
    assert_eq!(json["result"]["height"], 2);
    assert_eq!(json["result"]["channels"], 4);
    assert_eq!(json["result"]["has_alpha"], true);

    let _ = fs::remove_file(input);
}

#[test]
fn image_resize_width_only_preserves_aspect_ratio() {
    let input = temp_path("resize-input", "png");
    let output_path = temp_path("resize-output", "png");
    create_png(&input, 4, 2);

    let output = run(&[
        "image",
        "resize",
        input.to_str().unwrap(),
        "--width",
        "2",
        "-o",
        output_path.to_str().unwrap(),
        "--json",
    ]);
    let json = parse_stdout(&output);

    assert_eq!(json["operation"], "image.resize");
    assert_eq!(json["engine"]["id"], "raster-rs");
    assert_eq!(json["result"]["source_width"], 4);
    assert_eq!(json["result"]["source_height"], 2);
    assert_eq!(json["result"]["width"], 2);
    assert_eq!(json["result"]["height"], 1);

    let resized = image::open(&output_path).expect("resized image should decode");
    assert_eq!(resized.width(), 2);
    assert_eq!(resized.height(), 1);

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output_path);
}

#[test]
fn image_resize_rejects_existing_output() {
    let input = temp_path("conflict-input", "png");
    let output_path = temp_path("conflict-output", "png");
    create_png(&input, 4, 2);
    create_png(&output_path, 1, 1);

    let output = run(&[
        "image",
        "resize",
        input.to_str().unwrap(),
        "--width",
        "2",
        "-o",
        output_path.to_str().unwrap(),
        "--json",
    ]);

    assert_eq!(output.status.code(), Some(2));
    let json: Value =
        serde_json::from_slice(&output.stderr).expect("stderr should be structured JSON");
    assert_eq!(json["error"]["code"], "OUTPUT_CONFLICT");

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output_path);
}

#[test]
fn explicit_incompatible_engine_returns_structured_error() {
    let input = temp_path("engine-input", "png");
    create_png(&input, 4, 2);

    let output = run(&[
        "image",
        "info",
        input.to_str().unwrap(),
        "--engine",
        "yu-runtime",
        "--json",
    ]);

    assert_eq!(output.status.code(), Some(3));
    let json: Value =
        serde_json::from_slice(&output.stderr).expect("stderr should be structured JSON");
    assert_eq!(json["error"]["code"], "ENGINE_INCOMPATIBLE");

    let _ = fs::remove_file(input);
}
