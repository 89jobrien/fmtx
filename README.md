# fmtx

File extension to formatter dispatcher. Maps file extensions to formatter commands via a TOML
config and runs them — optionally in check mode for CI and pre-commit hooks.

## Usage

```bash
fmtx <file>           # format in place
fmtx --check <file>   # check only (non-zero exit if the file would change)
```

Files with no extension, or extensions not in config, exit 0 silently.

## Install

```bash
cargo install --path .
```

## Config

`~/.config/fmtx/config.toml`

**Single command:**

```toml
[extensions.rs]
command = "rustfmt"
args = []
check_args = ["--check"]
```

**Multiple commands** (run in sequence, stop on first failure):

```toml
[extensions.rs]
commands = [
  { command = "rustfmt", args = [], check_args = ["--check"] },
  { command = "my-linter", args = [], check_args = [] },
]
```

`args` are always passed. `check_args` are appended only when `--check` is given. The file path
is always the final argument. Extension keys are case-insensitive.

### Example full config

```toml
[extensions.rs]
command = "rustfmt"
check_args = ["--check"]

[extensions.sh]
command = "shfmt"
args = ["-w"]
check_args = ["-d"]

[extensions.py]
command = "ruff"
args = ["format"]
check_args = ["format", "--check"]

[extensions.toml]
command = "taplo"
args = ["format"]
check_args = ["format", "--check"]

[extensions.md]
command = "dprint"
args = ["fmt"]
check_args = ["check"]
```

## As a Claude Code hook

In `~/.claude/settings.json`, replace per-extension shell conditionals with a single hook:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          { "type": "command", "command": "fmtx \"$CLAUDE_FILE_PATH\"" }
        ]
      }
    ]
  }
}
```

## Exit codes

| Condition | Exit code |
|-----------|-----------|
| Formatted successfully | 0 |
| No extension / extension not configured | 0 |
| Config file missing or invalid | 1 |
| Formatter exited non-zero | 1 |
| Formatter killed by signal | 1 |
