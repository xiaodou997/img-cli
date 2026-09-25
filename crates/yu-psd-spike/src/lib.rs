use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Component, Path};
use std::time::Instant;

pub const CORPUS_SCHEMA_VERSION: &str = "1";
pub const REPORT_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsdCorpus {
    pub schema_version: String,
    pub fixtures: Vec<PsdFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsdFixture {
    pub id: String,
    pub path: String,
    pub format: PsdFormat,
    pub features: Vec<FixtureFeature>,
    pub expected: FixtureExpectation,
    pub provenance: FixtureProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PsdFormat {
    Psd,
    Psb,
}

impl PsdFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Psd => "psd",
            Self::Psb => "psb",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureFeature {
    SimplePixelLayers,
    NestedGroups,
    DuplicateLayerNames,
    TextLayers,
    Masks,
    BlendModes,
    Effects,
    SmartObjectMetadata,
    HighBitDepth,
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureExpectation {
    pub parse: ParseExpectation,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub layer_count: Option<usize>,
    #[serde(default)]
    pub minimum_tree_depth: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseExpectation {
    Accept,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureProvenance {
    pub kind: FixtureProvenanceKind,
    pub source: String,
    #[serde(default)]
    pub license: Option<String>,
    pub redistributable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureProvenanceKind {
    Synthetic,
    Redistributable,
    LocalOnly,
}

#[derive(Debug)]
pub struct CorpusError {
    message: String,
}

impl CorpusError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CorpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CorpusError {}

pub fn load_corpus(path: &Path) -> Result<PsdCorpus, CorpusError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CorpusError::new(format!(
            "failed to read PSD corpus {}: {error}",
            path.display()
        ))
    })?;
    let corpus = serde_json::from_str::<PsdCorpus>(&content).map_err(|error| {
        CorpusError::new(format!(
            "failed to parse PSD corpus {}: {error}",
            path.display()
        ))
    })?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    validate_corpus(&corpus, root)?;
    Ok(corpus)
}

pub fn validate_corpus(corpus: &PsdCorpus, root: &Path) -> Result<(), CorpusError> {
    if corpus.schema_version != CORPUS_SCHEMA_VERSION {
        return Err(CorpusError::new(format!(
            "unsupported PSD corpus schema {}; expected {}",
            corpus.schema_version, CORPUS_SCHEMA_VERSION
        )));
    }

    if corpus.fixtures.is_empty() {
        return Err(CorpusError::new(
            "PSD corpus must contain at least one fixture",
        ));
    }

    let mut ids = HashSet::new();
    for fixture in &corpus.fixtures {
        if fixture.id.trim().is_empty() {
            return Err(CorpusError::new("PSD fixture ID must not be empty"));
        }
        if !ids.insert(fixture.id.as_str()) {
            return Err(CorpusError::new(format!(
                "duplicate PSD fixture ID: {}",
                fixture.id
            )));
        }
        if fixture.features.is_empty() {
            return Err(CorpusError::new(format!(
                "PSD fixture {} must declare at least one feature",
                fixture.id
            )));
        }

        let relative = Path::new(&fixture.path);
        if fixture.path.trim().is_empty()
            || relative.is_absolute()
            || relative
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
        {
            return Err(CorpusError::new(format!(
                "PSD fixture {} uses an unsafe path: {}",
                fixture.id, fixture.path
            )));
        }

        let extension = relative
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !extension.eq_ignore_ascii_case(fixture.format.extension()) {
            return Err(CorpusError::new(format!(
                "PSD fixture {} format/path mismatch: expected .{}",
                fixture.id,
                fixture.format.extension()
            )));
        }

        let resolved = root.join(relative);
        if !resolved.is_file() {
            return Err(CorpusError::new(format!(
                "PSD fixture {} is missing: {}",
                fixture.id,
                resolved.display()
            )));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDescriptor {
    pub id: String,
    pub display_name: String,
    pub runtime: CandidateRuntime,
    pub status: CandidateStatus,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateRuntime {
    RustNative,
    Python,
    TypeScriptNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus {
    Skeleton,
    Wired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AdapterObservation {
    pub parse_success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_tree_depth: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterErrorKind {
    Unavailable,
    Execution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterError {
    pub kind: AdapterErrorKind,
    pub message: String,
}

impl AdapterError {
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::Unavailable,
            message: message.into(),
        }
    }

    pub fn execution(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::Execution,
            message: message.into(),
        }
    }
}

pub trait PsdCandidateAdapter {
    fn descriptor(&self) -> CandidateDescriptor;

    fn inspect(
        &self,
        input: &Path,
        fixture: &PsdFixture,
    ) -> Result<AdapterObservation, AdapterError>;
}

#[derive(Debug, Clone)]
struct SkeletonAdapter {
    descriptor: CandidateDescriptor,
}

impl SkeletonAdapter {
    fn new(
        id: &str,
        display_name: &str,
        runtime: CandidateRuntime,
        notes: &str,
    ) -> SkeletonAdapter {
        Self {
            descriptor: CandidateDescriptor {
                id: id.to_owned(),
                display_name: display_name.to_owned(),
                runtime,
                status: CandidateStatus::Skeleton,
                notes: notes.to_owned(),
            },
        }
    }
}

impl PsdCandidateAdapter for SkeletonAdapter {
    fn descriptor(&self) -> CandidateDescriptor {
        self.descriptor.clone()
    }

    fn inspect(
        &self,
        _input: &Path,
        _fixture: &PsdFixture,
    ) -> Result<AdapterObservation, AdapterError> {
        Err(AdapterError::unavailable(format!(
            "{} is a PR #10 adapter skeleton and is not wired to an engine yet",
            self.descriptor.id
        )))
    }
}

pub fn candidate_adapters() -> Vec<Box<dyn PsdCandidateAdapter>> {
    vec![
        Box::new(SkeletonAdapter::new(
            "rust-native",
            "Rust-native PSD candidate",
            CandidateRuntime::RustNative,
            "Reserved for Rust PSD implementations evaluated during M3.",
        )),
        Box::new(SkeletonAdapter::new(
            "psd-tools",
            "psd-tools candidate",
            CandidateRuntime::Python,
            "Reserved for the Python psd-tools compatibility candidate.",
        )),
        Box::new(SkeletonAdapter::new(
            "typescript-psd",
            "TypeScript PSD candidate",
            CandidateRuntime::TypeScriptNode,
            "Reserved for TypeScript/Node PSD implementations evaluated during M3.",
        )),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateReport {
    pub schema_version: String,
    pub candidate: CandidateDescriptor,
    pub fixtures: Vec<FixtureReport>,
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureReport {
    pub fixture_id: String,
    pub status: FixtureStatus,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<AdapterObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReportSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
}

pub fn run_candidate(
    corpus_path: &Path,
    adapter: &dyn PsdCandidateAdapter,
) -> Result<CandidateReport, CorpusError> {
    let corpus = load_corpus(corpus_path)?;
    let root = corpus_path.parent().unwrap_or_else(|| Path::new("."));
    let mut fixtures = Vec::with_capacity(corpus.fixtures.len());

    for fixture in &corpus.fixtures {
        let input = root.join(&fixture.path);
        let started = Instant::now();
        let result = adapter.inspect(&input, fixture);
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

        let report = match result {
            Ok(observation) => {
                let issues = compare_observation(fixture, &observation);
                if issues.is_empty() {
                    FixtureReport {
                        fixture_id: fixture.id.clone(),
                        status: FixtureStatus::Passed,
                        duration_ms,
                        observation: Some(observation),
                        diagnostic: None,
                    }
                } else {
                    FixtureReport {
                        fixture_id: fixture.id.clone(),
                        status: FixtureStatus::Failed,
                        duration_ms,
                        observation: Some(observation),
                        diagnostic: Some(issues.join("; ")),
                    }
                }
            }
            Err(error) => FixtureReport {
                fixture_id: fixture.id.clone(),
                status: match error.kind {
                    AdapterErrorKind::Unavailable => FixtureStatus::Skipped,
                    AdapterErrorKind::Execution => FixtureStatus::Error,
                },
                duration_ms,
                observation: None,
                diagnostic: Some(error.message),
            },
        };

        fixtures.push(report);
    }

    let summary = summarize(&fixtures);
    Ok(CandidateReport {
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        candidate: adapter.descriptor(),
        fixtures,
        summary,
    })
}

fn compare_observation(fixture: &PsdFixture, observation: &AdapterObservation) -> Vec<String> {
    let expected_parse = matches!(fixture.expected.parse, ParseExpectation::Accept);
    if observation.parse_success != expected_parse {
        return vec![format!(
            "parse expectation mismatch: expected {}, observed {}",
            fixture.expected.parse.as_str(),
            if observation.parse_success {
                "accept"
            } else {
                "reject"
            }
        )];
    }

    if !observation.parse_success {
        return Vec::new();
    }

    let mut issues = Vec::new();
    compare_optional("width", fixture.expected.width, observation.width, &mut issues);
    compare_optional(
        "height",
        fixture.expected.height,
        observation.height,
        &mut issues,
    );
    compare_optional(
        "layer_count",
        fixture.expected.layer_count,
        observation.layer_count,
        &mut issues,
    );
    compare_optional(
        "minimum_tree_depth",
        fixture.expected.minimum_tree_depth,
        observation.maximum_tree_depth,
        &mut issues,
    );

    issues
}

impl ParseExpectation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }
}

fn compare_optional<T>(
    field: &str,
    expected: Option<T>,
    observed: Option<T>,
    issues: &mut Vec<String>,
) where
    T: Copy + PartialEq + fmt::Display,
{
    let Some(expected) = expected else {
        return;
    };
    match observed {
        Some(observed) if observed == expected => {}
        Some(observed) => issues.push(format!(
            "{field} mismatch: expected {expected}, observed {observed}"
        )),
        None => issues.push(format!("{field} missing: expected {expected}")),
    }
}

fn summarize(fixtures: &[FixtureReport]) -> ReportSummary {
    let mut summary = ReportSummary::default();
    for fixture in fixtures {
        match fixture.status {
            FixtureStatus::Passed => summary.passed += 1,
            FixtureStatus::Failed => summary.failed += 1,
            FixtureStatus::Skipped => summary.skipped += 1,
            FixtureStatus::Error => summary.errors += 1,
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn committed_corpus_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/psd/corpus.json")
    }

    struct RejectingAdapter;

    impl PsdCandidateAdapter for RejectingAdapter {
        fn descriptor(&self) -> CandidateDescriptor {
            CandidateDescriptor {
                id: "test-rejecting".to_owned(),
                display_name: "Test rejecting adapter".to_owned(),
                runtime: CandidateRuntime::RustNative,
                status: CandidateStatus::Wired,
                notes: "Test-only adapter.".to_owned(),
            }
        }

        fn inspect(
            &self,
            _input: &Path,
            _fixture: &PsdFixture,
        ) -> Result<AdapterObservation, AdapterError> {
            Ok(AdapterObservation {
                parse_success: false,
                ..AdapterObservation::default()
            })
        }
    }

    #[test]
    fn committed_corpus_is_valid() {
        let corpus = load_corpus(&committed_corpus_path()).expect("committed corpus should be valid");
        assert_eq!(corpus.schema_version, CORPUS_SCHEMA_VERSION);
        assert_eq!(corpus.fixtures.len(), 1);
    }

    #[test]
    fn expected_reject_is_a_passing_conformance_result() {
        let report = run_candidate(&committed_corpus_path(), &RejectingAdapter)
            .expect("test adapter should run");
        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.summary.failed, 0);
        assert_eq!(report.summary.errors, 0);
    }

    #[test]
    fn candidate_skeletons_skip_without_selecting_an_engine() {
        let fixture_count = load_corpus(&committed_corpus_path())
            .expect("committed corpus should load")
            .fixtures
            .len();

        for adapter in candidate_adapters() {
            let report = run_candidate(&committed_corpus_path(), adapter.as_ref())
                .expect("skeleton adapter should produce a report");
            assert_eq!(report.candidate.status, CandidateStatus::Skeleton);
            assert_eq!(report.summary.skipped, fixture_count);
            assert_eq!(report.summary.failed, 0);
            assert_eq!(report.summary.errors, 0);
        }
    }

    #[test]
    fn corpus_rejects_parent_directory_traversal() {
        let corpus = PsdCorpus {
            schema_version: CORPUS_SCHEMA_VERSION.to_owned(),
            fixtures: vec![PsdFixture {
                id: "escape".to_owned(),
                path: "../escape.psd".to_owned(),
                format: PsdFormat::Psd,
                features: vec![FixtureFeature::Malformed],
                expected: FixtureExpectation {
                    parse: ParseExpectation::Reject,
                    width: None,
                    height: None,
                    layer_count: None,
                    minimum_tree_depth: None,
                },
                provenance: FixtureProvenance {
                    kind: FixtureProvenanceKind::Synthetic,
                    source: "test".to_owned(),
                    license: None,
                    redistributable: true,
                },
            }],
        };

        let error = validate_corpus(&corpus, Path::new(".")).expect_err("path must be rejected");
        assert!(error.to_string().contains("unsafe path"));
    }
}
