//! Reading tests driven by the fixtures in tests/fixtures.
//!
//! The fixtures and the expectations they are checked against are produced by
//! scripts/make_fixtures.py, which writes the format from its documentation
//! and never calls this crate. Regenerate them with:
//!
//!     python scripts/make_fixtures.py

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use renzip_core::{Archive, Layout, Selection};

/// Directory holding the generated fixtures.
fn fixtures() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    assert!(
        dir.join("expected.json").is_file(),
        "fixtures are missing, run: python scripts/make_fixtures.py"
    );
    dir.canonicalize().expect("canonical fixtures directory")
}

/// The expectations file, as a map of archive name to its description.
fn expectations() -> serde_json::Map<String, serde_json::Value> {
    let raw =
        std::fs::read_to_string(fixtures().join("expected.json")).expect("read expected.json");
    serde_json::from_str::<serde_json::Value>(&raw)
        .expect("parse expected.json")
        .as_object()
        .expect("expected.json is an object")
        .clone()
}

/// Expected members of one archive, as name to content.
fn members(description: &serde_json::Value) -> BTreeMap<String, Vec<u8>> {
    description["entries"]
        .as_object()
        .expect("entries is an object")
        .iter()
        .map(|(name, hex)| {
            let hex = hex.as_str().expect("content is a hex string");
            let bytes = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
                .collect();
            (name.clone(), bytes)
        })
        .collect()
}

/// Names listed under a key of the description, empty when absent.
fn names(description: &serde_json::Value, key: &str) -> Vec<String> {
    description
        .get(key)
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .map(|v| v.as_str().expect("a name").to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_fixture_lists_and_reads_exactly_what_it_holds() {
    for (name, description) in expectations() {
        let mut archive = Archive::open(fixtures().join(&name))
            .unwrap_or_else(|e| panic!("{name} should open: {e}"));

        assert_eq!(
            archive.version().as_str(),
            description["version"].as_str().unwrap(),
            "{name}: version"
        );

        let expected = members(&description);
        let listed: Vec<String> = archive
            .entries()
            .iter()
            .map(|entry| entry.path.clone())
            .collect();
        assert_eq!(
            listed,
            expected.keys().cloned().collect::<Vec<_>>(),
            "{name}: listed entries"
        );

        let total: u64 = expected.values().map(|content| content.len() as u64).sum();
        assert_eq!(archive.total_size(), total, "{name}: total size");

        for (path, content) in &expected {
            assert_eq!(
                &archive.read_entry(path).unwrap(),
                content,
                "{name}: {path}"
            );
        }

        let mut rejected: Vec<String> = archive.rejected().iter().map(|r| r.path.clone()).collect();
        rejected.sort();
        assert_eq!(
            rejected,
            names(&description, "rejected"),
            "{name}: rejected"
        );
        assert_eq!(
            archive.multi_segment_paths(),
            names(&description, "multi_segment"),
            "{name}: multi segment"
        );
    }
}

#[test]
fn extraction_restores_bytes_and_tree() {
    for name in ["basic-3.0.rpa", "prefix-3.0.rpa", "legacy-1.0.rpa"] {
        let description = expectations()[name].clone();
        let expected = members(&description);
        let dest = tempfile::tempdir().expect("temp dir");

        let mut archive = Archive::open(fixtures().join(name)).unwrap();
        let report = archive
            .extract(Selection::All, dest.path(), Layout::Tree, &mut |_| true)
            .unwrap();

        assert_eq!(report.extracted, expected.len(), "{name}: extracted count");
        assert!(report.skipped.is_empty(), "{name}: {:?}", report.skipped);
        assert!(!report.cancelled, "{name}: cancelled");

        for (path, content) in &expected {
            let written = dest
                .path()
                .join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
            assert_eq!(
                &std::fs::read(&written).unwrap_or_else(|e| panic!("{name}: {path}: {e}")),
                content,
                "{name}: {path} content"
            );
        }
    }
}

#[test]
fn hostile_paths_never_reach_the_file_system() {
    let root = tempfile::tempdir().expect("temp dir");
    let dest = root.path().join("out");

    let mut archive = Archive::open(fixtures().join("malicious-3.0.rpa")).unwrap();
    let report = archive
        .extract(Selection::All, &dest, Layout::Tree, &mut |_| true)
        .unwrap();

    assert_eq!(report.extracted, 1);
    assert!(dest.join("harmless.txt").is_file());

    let siblings: Vec<String> = std::fs::read_dir(root.path())
        .unwrap()
        .map(|item| item.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        siblings,
        vec!["out".to_string()],
        "nothing escaped {dest:?}"
    );

    let written: Vec<String> = std::fs::read_dir(&dest)
        .unwrap()
        .map(|item| item.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(written, vec!["harmless.txt".to_string()]);
}

#[test]
fn an_entry_reaching_past_the_end_is_reported_not_read() {
    let archive = Archive::open(fixtures().join("out-of-bounds-3.0.rpa")).unwrap();
    assert_eq!(archive.entries().len(), 1);
    assert_eq!(archive.rejected().len(), 1);
    assert!(
        archive.rejected()[0].reason.contains("byte archive"),
        "{:?}",
        archive.rejected()[0]
    );
}

#[test]
fn a_truncated_archive_fails_to_open() {
    let error = Archive::open(fixtures().join("truncated-3.0.rpa")).unwrap_err();
    assert!(
        matches!(error, renzip_core::Error::Truncated(_)),
        "unexpected error: {error}"
    );
}

#[test]
fn a_file_that_is_not_an_archive_fails_to_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("not-an-archive.rpa");
    std::fs::write(&path, b"PK\x03\x04 this is a zip, actually").unwrap();

    let error = Archive::open(&path).unwrap_err();
    assert!(
        matches!(error, renzip_core::Error::UnsupportedHeader(_)),
        "unexpected error: {error}"
    );
}

#[test]
fn a_selection_extracts_only_what_it_names() {
    let dest = tempfile::tempdir().unwrap();
    let mut archive = Archive::open(fixtures().join("basic-2.0.rpa")).unwrap();

    let report = archive
        .extract(
            Selection::Paths(&["script.rpy".to_string()]),
            dest.path(),
            Layout::Tree,
            &mut |_| true,
        )
        .unwrap();

    assert_eq!(report.extracted, 1);
    assert!(dest.path().join("script.rpy").is_file());
    assert!(!dest.path().join("images").exists());

    let unknown = archive.extract(
        Selection::Paths(&["nope.rpy".to_string()]),
        dest.path(),
        Layout::Tree,
        &mut |_| true,
    );
    assert!(matches!(
        unknown.unwrap_err(),
        renzip_core::Error::EntryNotFound(_)
    ));
}

#[test]
fn cancelling_stops_after_the_current_entry() {
    let dest = tempfile::tempdir().unwrap();
    let mut archive = Archive::open(fixtures().join("basic-3.0.rpa")).unwrap();
    let total = archive.entries().len();
    assert!(total > 1, "the fixture needs more than one entry");

    let mut seen = 0;
    let report = archive
        .extract(Selection::All, dest.path(), Layout::Tree, &mut |progress| {
            seen += 1;
            assert_eq!(progress.total, total);
            false
        })
        .unwrap();

    assert!(report.cancelled);
    assert_eq!(seen, 1);
    assert_eq!(report.extracted, 1);
}
