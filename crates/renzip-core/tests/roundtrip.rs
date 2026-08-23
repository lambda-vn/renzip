//! Loop test: build an archive, open it, extract it, compare byte for byte.
//!
//! This is the test that catches an off-by-one on a prefix or on a byte
//! range: a wrong length shows up as a file that differs from its original by
//! a few bytes, which a test that only counts entries would let through.

use std::collections::BTreeMap;
use std::path::Path;

use renzip_core::{Archive, Layout, Selection, Version};

/// Files written into the source tree of every build test.
fn sources() -> BTreeMap<&'static str, Vec<u8>> {
    BTreeMap::from([
        (
            "script.rpy",
            b"label start:\n    \"Bonjour.\"\n    return\n".to_vec(),
        ),
        ("images/bg.png", (0u16..=511).map(|b| b as u8).collect()),
        ("gui/nested/deep/notes.txt", b"deep\n".to_vec()),
        ("empty.dat", Vec::new()),
    ])
}

/// Materialize [`sources`] under `dir`.
fn write_sources(dir: &Path) {
    for (name, content) in sources() {
        let path = dir.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
}

#[test]
fn build_open_extract_returns_identical_files() {
    for version in [Version::Rpa2_0, Version::Rpa3_0] {
        let work = tempfile::tempdir().unwrap();
        let source = work.path().join("source");
        let dest = work.path().join("out");
        let archive_path = work.path().join("built.rpa");
        write_sources(&source);

        let build = renzip_core::build(
            &source,
            &archive_path,
            version,
            renzip_core::DEFAULT_KEY,
            &mut |_| true,
        )
        .unwrap();
        assert_eq!(build.entries, sources().len(), "{version}: stored count");
        assert!(build.skipped.is_empty(), "{version}: {:?}", build.skipped);

        let mut archive = Archive::open(&archive_path).unwrap();
        assert_eq!(archive.version(), version);
        assert_eq!(
            archive
                .entries()
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            sources().keys().copied().collect::<Vec<_>>(),
            "{version}: listed entries"
        );

        let report = archive
            .extract(Selection::All, &dest, Layout::Tree, &mut |_| true)
            .unwrap();
        assert_eq!(report.extracted, sources().len());
        assert!(report.skipped.is_empty(), "{version}: {:?}", report.skipped);

        for (name, content) in sources() {
            let extracted =
                std::fs::read(dest.join(name.replace('/', std::path::MAIN_SEPARATOR_STR))).unwrap();
            assert_eq!(extracted, content, "{version}: {name}");
            assert_eq!(
                archive.read_entry(name).unwrap(),
                content,
                "{version}: {name}"
            );
        }
    }
}

#[test]
fn an_archive_this_crate_writes_is_read_back_by_python() {
    // The other half of the loop: reading our own archives only proves we
    // agree with ourselves. scripts/read_archive.py uses the pickle module
    // and the logic of renpy/loader.py, so agreeing with it is agreeing with
    // the engine. Skipped where no interpreter is installed.
    let Some(python) = python_command() else {
        eprintln!("skipped: no python interpreter on PATH");
        return;
    };

    let work = tempfile::tempdir().unwrap();
    let source = work.path().join("source");
    let archive_path = work.path().join("built.rpa");
    write_sources(&source);
    renzip_core::build(
        &source,
        &archive_path,
        Version::Rpa3_0,
        renzip_core::DEFAULT_KEY,
        &mut |_| true,
    )
    .unwrap();

    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/read_archive.py");
    let output = std::process::Command::new(python)
        .arg(script)
        .arg(&archive_path)
        .output()
        .expect("run read_archive.py");
    assert!(
        output.status.success(),
        "read_archive.py failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let seen: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(seen["version"], "RPA-3.0");
    for (name, content) in sources() {
        let expected: String = content.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(seen["entries"][name], expected, "{name} as python reads it");
    }
}

/// Name of an interpreter that answers on this machine, if any.
fn python_command() -> Option<&'static str> {
    ["python", "python3"].into_iter().find(|command| {
        std::process::Command::new(command)
            .arg("--version")
            .output()
            .is_ok_and(|out| out.status.success())
    })
}

#[test]
fn a_flat_extraction_drops_folders_and_refuses_to_overwrite_a_name_twice() {
    let work = tempfile::tempdir().unwrap();
    let source = work.path().join("source");
    let dest = work.path().join("flat");
    let archive_path = work.path().join("built.rpa");
    write_sources(&source);
    // A second file named notes.txt, in another folder: flattening makes the
    // two collide, and the loser has to be reported rather than dropped.
    std::fs::create_dir_all(source.join("gui")).unwrap();
    std::fs::write(source.join("gui").join("notes.txt"), b"shallow\n").unwrap();

    renzip_core::build(
        &source,
        &archive_path,
        Version::Rpa3_0,
        renzip_core::DEFAULT_KEY,
        &mut |_| true,
    )
    .unwrap();

    let mut archive = Archive::open(&archive_path).unwrap();
    let report = archive
        .extract(Selection::All, &dest, Layout::Flat, &mut |_| true)
        .unwrap();

    assert_eq!(
        report.extracted,
        sources().len(),
        "one of the two notes.txt is kept"
    );
    assert_eq!(report.skipped.len(), 1, "{:?}", report.skipped);
    assert!(report.skipped[0].path.ends_with("notes.txt"));
    assert!(
        report.skipped[0].reason.contains("already wrote"),
        "{:?}",
        report.skipped[0]
    );

    assert!(dest.join("bg.png").is_file(), "folders are gone");
    assert!(dest.join("notes.txt").is_file());
    assert!(!dest.join("images").exists());
}

#[test]
fn scan_lists_what_a_build_would_pack() {
    let work = tempfile::tempdir().unwrap();
    let source = work.path().join("source");
    write_sources(&source);

    let report = renzip_core::scan(&source).unwrap();
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_eq!(
        report
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        sources().keys().copied().collect::<Vec<_>>()
    );
    for file in &report.files {
        assert_eq!(
            file.size as usize,
            sources()[file.path.as_str()].len(),
            "{}",
            file.path
        );
    }

    assert!(renzip_core::scan(&work.path().join("nowhere")).is_err());
}

#[test]
fn an_archive_built_from_an_empty_directory_opens_empty() {
    let work = tempfile::tempdir().unwrap();
    let source = work.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    let archive_path = work.path().join("empty.rpa");

    let report = renzip_core::build(
        &source,
        &archive_path,
        Version::Rpa3_0,
        renzip_core::DEFAULT_KEY,
        &mut |_| true,
    )
    .unwrap();
    assert_eq!(report.entries, 0);

    let archive = Archive::open(&archive_path).unwrap();
    assert!(archive.entries().is_empty());
    assert_eq!(archive.total_size(), 0);
}

#[test]
fn versions_this_crate_cannot_write_are_refused_before_any_file_is_touched() {
    let work = tempfile::tempdir().unwrap();
    let source = work.path().join("source");
    write_sources(&source);
    let archive_path = work.path().join("nope.rpa");

    let error = renzip_core::build(
        &source,
        &archive_path,
        Version::Rpa3_2,
        renzip_core::DEFAULT_KEY,
        &mut |_| true,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        renzip_core::Error::UnsupportedWriteVersion(_)
    ));
    assert!(!archive_path.exists(), "no partial archive is left behind");
}
