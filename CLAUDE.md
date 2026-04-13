# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What it does

`fmtx` is a file extension-to-formatter dispatcher. Given a file path, it looks up the configured formatter for that file's extension and runs it. Supports a `--check` mode (for CI/pre-commit) that appends extra args to the formatter command instead of formatting in-place.

Usage: `fmtx [--check] <file>`

## Commands

```bash
cargo build                            # Build
cargo nextest run                      # Run tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Architecture

Four source files — no modules beyond `src/`:

| File | Role |
|------|------|
| `domain.rs` | Core types: `CommandStep`, `FormatterEntry { steps }`, `FormatterError`, `Formatter` trait, `extension_of` |
| `config.rs` | Loads `~/.config/fmtx/config.toml`, deserialises both single and multi-command forms into `HashMap<ext, FormatterEntry>` |
| `runner.rs` | `CommandRunner` implements `Formatter` — iterates `entry.steps`, stops on first failure |
| `main.rs` | Parses args, wires config → runner, maps errors to exit codes |

`FormatterEntry` holds a `Vec<CommandStep>`. Single-command config entries are normalised to a one-element vec on load, so the runner always sees the same shape.

## Config file format

`~/.config/fmtx/config.toml`:

```toml
# Single command (shorthand)
[extensions.rs]
command = "rustfmt"
args = []
check_args = ["--check"]

# Multiple commands (run in sequence, stop on first failure)
[extensions.rs]
commands = [
  { command = "rustfmt", args = [], check_args = ["--check"] },
  { command = "clippy-driver", args = [], check_args = [] },
]

[extensions.py]
command = "ruff"
args = ["format"]
check_args = ["format", "--check"]
```

Extension keys are normalised to lowercase. `args` are always passed; `format_args` are appended only in format mode (not `--check`); `check_args` are appended only in `--check` mode. The file path is always the final argument to each command.

## Config path

On macOS, `dirs_next::config_dir()` resolves to `~/Library/Application Support/fmtx/config.toml`,
**not** `~/.config/`. Override with `FMTX_CONFIG=/path/to/config.toml` — integration tests use
this to avoid depending on the live dotfile.

## Gotchas

- LSP diagnostics go stale after edits — always verify with `cargo nextest run`, not the IDE error list.

## Key behaviours

- Missing extension → `ExitCode::SUCCESS` (nothing to do, not an error)
- Extension not in config → `ExitCode::SUCCESS` (unconfigured, not an error)
- Formatter non-zero exit → `ExitCode::FAILURE` with the exit code in the message
- Formatter killed by signal → `ExitCode::FAILURE` with "terminated by a signal"
