use assert_cmd::Command;
use std::fs;
use tempfile::Builder;

fn fmtx() -> Command {
    let mut cmd = Command::cargo_bin("fmtx").unwrap();
    // Point at the known config so tests work regardless of platform config dir.
    cmd.env(
        "FMTX_CONFIG",
        dirs_next::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(std::env::var("HOME").unwrap()))
            .join("fmtx")
            .join("config.toml"),
    );
    cmd
}

// ============================================================================
// .rs — rustfmt
// ============================================================================

const RS_UNFORMATTED: &str = "fn main(){println!(\"hi\");}";
const RS_FORMATTED: &str = "fn main() {\n    println!(\"hi\");\n}\n";

#[test]
fn rs_format_rewrites_file() {
    let f = Builder::new().suffix(".rs").tempfile().unwrap();
    fs::write(f.path(), RS_UNFORMATTED).unwrap();
    fmtx().arg(f.path()).assert().success();
    assert_eq!(fs::read_to_string(f.path()).unwrap(), RS_FORMATTED);
}

#[test]
fn rs_check_passes_on_formatted() {
    let f = Builder::new().suffix(".rs").tempfile().unwrap();
    fs::write(f.path(), RS_FORMATTED).unwrap();
    fmtx().arg("--check").arg(f.path()).assert().success();
}

#[test]
fn rs_check_fails_on_unformatted() {
    let f = Builder::new().suffix(".rs").tempfile().unwrap();
    fs::write(f.path(), RS_UNFORMATTED).unwrap();
    fmtx().arg("--check").arg(f.path()).assert().failure();
}

// ============================================================================
// .sh — shfmt
// ============================================================================

const SH_UNFORMATTED: &str = "#!/bin/sh\nfoo(){\necho hi\n}\n";
const SH_FORMATTED: &str = "#!/bin/sh\nfoo() {\n\techo hi\n}\n";

#[test]
fn sh_format_rewrites_file() {
    let f = Builder::new().suffix(".sh").tempfile().unwrap();
    fs::write(f.path(), SH_UNFORMATTED).unwrap();
    fmtx().arg(f.path()).assert().success();
    assert_eq!(fs::read_to_string(f.path()).unwrap(), SH_FORMATTED);
}

#[test]
fn sh_check_passes_on_formatted() {
    let f = Builder::new().suffix(".sh").tempfile().unwrap();
    fs::write(f.path(), SH_FORMATTED).unwrap();
    fmtx().arg("--check").arg(f.path()).assert().success();
}

#[test]
fn sh_check_fails_on_unformatted() {
    let f = Builder::new().suffix(".sh").tempfile().unwrap();
    fs::write(f.path(), SH_UNFORMATTED).unwrap();
    fmtx().arg("--check").arg(f.path()).assert().failure();
}

// ============================================================================
// .toml — taplo
// Temp files placed in system temp dir to avoid repo-level taplo config
// treating the file as excluded.
// ============================================================================

const TOML_UNFORMATTED: &str = "[package]\n  name   =   \"test\"\n";
const TOML_FORMATTED: &str = "[package]\nname = \"test\"\n";

#[test]
fn toml_format_rewrites_file() {
    let f = Builder::new()
        .suffix(".toml")
        .tempfile_in(std::env::temp_dir())
        .unwrap();
    fs::write(f.path(), TOML_UNFORMATTED).unwrap();
    fmtx().arg(f.path()).assert().success();
    assert_eq!(fs::read_to_string(f.path()).unwrap(), TOML_FORMATTED);
}

#[test]
fn toml_check_passes_on_formatted() {
    let f = Builder::new()
        .suffix(".toml")
        .tempfile_in(std::env::temp_dir())
        .unwrap();
    fs::write(f.path(), TOML_FORMATTED).unwrap();
    fmtx().arg("--check").arg(f.path()).assert().success();
}

#[test]
fn toml_check_fails_on_unformatted() {
    let f = Builder::new()
        .suffix(".toml")
        .tempfile_in(std::env::temp_dir())
        .unwrap();
    fs::write(f.path(), TOML_UNFORMATTED).unwrap();
    fmtx().arg("--check").arg(f.path()).assert().failure();
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn no_extension_exits_success() {
    let f = Builder::new().tempfile().unwrap();
    fmtx().arg(f.path()).assert().success();
}

#[test]
fn unconfigured_extension_exits_success() {
    let f = Builder::new().suffix(".xyz").tempfile().unwrap();
    fmtx().arg(f.path()).assert().success();
}
