use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn rmcm() -> Command {
    cargo_bin_cmd!("rmcm")
}

#[test]
fn test_help() {
    rmcm().arg("--help").assert().success();
}

#[test]
fn test_version() {
    rmcm().arg("--version").assert().success();
}

// ========== Stdin ==========

#[test]
fn test_stdin_requires_language() {
    rmcm()
        .write_stdin("// test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Language must be specified"));
}

#[cfg(feature = "rust-lang")]
#[test]
fn test_stdin_with_language() {
    let input = "// a comment\nfn main() {}";
    let expected = "\nfn main() {}";

    rmcm()
        .args(&["--language", "rust"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(expected);
}

#[cfg(feature = "rust-lang")]
#[test]
fn test_stdin_with_json() {
    let input = "// comment";
    rmcm()
        .args(&["--language", "rust", "--json"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""result":"#));
}

// ========== Single file ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_single_file_to_stdout() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "// comment\nfn main() {}").unwrap();

    rmcm()
        .arg(file_path)
        .assert()
        .success()
        .stdout("\nfn main() {}\n");
}

#[cfg(feature = "rust-lang")]
#[test]
fn test_single_file_in_place() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "// comment\nfn main() {}").unwrap();

    rmcm()
        .args(&["--in-place", file_path.to_str().unwrap()])
        .assert()
        .success();

    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "\nfn main() {}\n");
}

// ========== Output directory ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_output_dir() {
    let dir = tempdir().unwrap();
    let input_file = dir.path().join("sub/test.rs");
    fs::create_dir_all(input_file.parent().unwrap()).unwrap();
    fs::write(&input_file, "// comment\nfn main() {}").unwrap();

    let out_dir = dir.path().join("out");
    rmcm()
        .args(&[
            "--output-dir",
            out_dir.to_str().unwrap(),
            input_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    let out_file = out_dir.join("sub/test.rs");
    assert!(out_file.exists());
    let content = fs::read_to_string(out_file).unwrap();
    assert_eq!(content, "\nfn main() {}\n");
}

// ========== Recursive ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_recursive() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "// a\nfn a() {}").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/b.rs"), "// b\nfn b() {}").unwrap();

    let out_dir = dir.path().join("out");
    rmcm()
        .args(&[
            "--recursive",
            "--output-dir",
            out_dir.to_str().unwrap(),
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let out_a = out_dir.join("a.rs");
    let out_b = out_dir.join("sub/b.rs");
    assert!(out_a.exists());
    assert!(out_b.exists());

    let content_a = fs::read_to_string(out_a).unwrap();
    let content_b = fs::read_to_string(out_b).unwrap();
    assert_eq!(content_a, "\nfn a() {}\n");
    assert_eq!(content_b, "\nfn b() {}\n");
}

// ========== Collapse whitespace ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_collapse_whitespace() {
    let input = "fn main() {\n    // comment\n\n\n    println!();\n}\n";
    let expected = "fn main() {\n\n    println!();\n}\n";

    rmcm()
        .args(&["--language", "rust", "--collapse-whitespace", "1"])
        .write_stdin(input)
        .assert()
        .success()
        .stdout(expected);
}

// ========== Dry run dan diff ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_dry_run() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "// comment").unwrap();

    rmcm()
        .args(&["--dry-run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("DRY RUN"));
}

#[cfg(feature = "rust-lang")]
#[test]
fn test_diff() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "// comment\nfn main() {}").unwrap();

    rmcm()
        .args(&["--diff", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Diff for"));
}

// ========== JSON output (summary) ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_json_summary() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("ok.rs"), "// ok").unwrap();
    fs::write(dir.path().join("bad.xyz"), "bad").unwrap();

    let assert = rmcm()
        .args(&["--json", "--force", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["success"], 1);
    assert_eq!(json["failed"], 1);
    assert_eq!(json["failures"].as_array().unwrap().len(), 1);
}

// ========== Config file ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_config_file() {
    let dir = tempdir().unwrap();
    let config = r#"
        language = "rust"
        collapse_whitespace = 1
        recursive = true
    "#;
    let config_path = dir.path().join("rmcm.toml");
    fs::write(&config_path, config).unwrap();

    fs::write(dir.path().join("a.rs"), "// a\n\n\nfn a() {}").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/b.rs"), "// b\n\n\nfn b() {}").unwrap();

    let out_dir = dir.path().join("out");
    rmcm()
        .args(&[
            "--config",
            config_path.to_str().unwrap(),
            "--output-dir",
            out_dir.to_str().unwrap(),
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let out_a = out_dir.join("a.rs");
    let out_b = out_dir.join("sub/b.rs");
    assert!(out_a.exists());
    assert!(out_b.exists());

    let content_a = fs::read_to_string(out_a).unwrap();
    let content_b = fs::read_to_string(out_b).unwrap();
    assert!(!content_a.contains("//"));
    assert!(!content_b.contains("//"));
}

// ========== Force continue on error ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_force_continues_on_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("good.rs"), "// good").unwrap();
    fs::write(dir.path().join("bad.unknown"), "content").unwrap();

    rmcm()
        .args(&[dir.path().to_str().unwrap()])
        .assert()
        .failure();

    let assert = rmcm()
        .args(&["--force", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let output = assert.get_output();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        stderr.contains("Processed: 1, Failed: 1")
            || stderr.contains("Successfully processed 1 files")
    );
}

// ========== Multiple files to stdout error ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_multiple_files_to_stdout_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "").unwrap();
    fs::write(dir.path().join("b.rs"), "").unwrap();

    rmcm()
        .args(&[dir.path().join("a.rs"), dir.path().join("b.rs")])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot output multiple files to stdout",
        ));
}

// ========== Directory without recursive error ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_directory_without_recursive_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.rs"), "").unwrap();

    rmcm()
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Directory not allowed"));
}

// ========== Language override dan deteksi ==========

#[cfg(feature = "rust-lang")]
#[test]
fn test_language_override() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "// comment").unwrap();

    rmcm()
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported language"));

    rmcm()
        .args(&["--language", "rust", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout("\n");
}

#[cfg(feature = "rust-lang")]
#[test]
fn test_detection_from_extension() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("program.rs");
    fs::write(&file_path, "// rust comment").unwrap();

    rmcm().arg(&file_path).assert().success().stdout("\n");
}
