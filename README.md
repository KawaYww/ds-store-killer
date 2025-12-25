# dsk

> F**king .DS_Store ! ! ! ! !
> ?
> Kill those pesky `.DS_Store` files on macOS

**macOS only** — Linux/Windows folks, you don't have to deal with this crap.

## Features

- Interactive confirmation (or `-y` to skip)
- Dry-run mode (`-n`)
- Git safety — skips tracked files by default, `--force` to override
- Recursive or single-dir
- Daemon mode with `--serve`
- launchd service for auto-start on boot
- Exclude patterns (`-e node_modules -e .git`)
- Fast parallel scanning via `jwalk`

## Install

```bash
cargo install ds-store-killer
```

Nix users:
```bash
nix run github:kawayww/ds-store-killer

# or add to flake
{
  inputs.dsk.url = "github:kawayww/ds-store-killer";
}
```

Build from source:
```bash
git clone https://github.com/kawayww/ds-store-killer
cd ds-store-killer
cargo build --release
```

## Usage

```bash
dsk                      # current dir, asks before deleting
dsk -y                   # just delete, don't ask
dsk -n                   # dry-run, scan only
dsk -r ~/Projects        # recursive
dsk -r --force ~/repo    # include git-tracked files
dsk -ry --stats .        # recursive, no confirm, show timing
dsk -ry -e node_modules  # exclude pattern
dsk --serve ~/Desktop    # daemon, auto-kill new ones
```

## Git Safety

Deleting git-tracked `.DS_Store` messes up your commit history. By default, `dsk` skips them.

| Mode | Behavior |
|------|----------|
| Default | Skips git-tracked files |
| `--force` | Includes them (still asks for confirmation) |

```bash
dsk -r ~/my-repo           # skips tracked files
dsk -r --force ~/my-repo   # includes them, still confirms
dsk -ry --force ~/my-repo  # includes them, no confirmation
```

Why would `.DS_Store` be tracked?
- Someone forgot `.gitignore`
- Force-added with `git add -f`
- `.gitignore` was added after the fact

## Cache

Non-recursive scans are cached in `$TMPDIR/dsk-cache/`. Auto-invalidates when directory changes.

Recursive mode doesn't cache — can't reliably detect subdirectory changes, so we rescan every time.

## Daemon Mode

Watch directories and auto-delete new `.DS_Store` as they appear:

```bash
dsk --serve              # current dir
dsk --serve ~/Desktop    # specific dir
dsk --serve . -e .git    # with exclusions
```

Runs in foreground, Ctrl+C to stop.

## launchd Service

For background monitoring that survives reboots:

```bash
dsk service install                       # watch ~ by default
dsk service install ~/Desktop ~/Projects  # multi-path
dsk service start
dsk service status
dsk service stop
dsk service uninstall
```

Uses kqueue — watching 10 dirs costs the same as watching 1.

Logs: `/tmp/dsk.out.log`, `/tmp/dsk.err.log`

## CLI Reference

```
Usage: dsk [OPTIONS] [PATH] [COMMAND]

Commands:
  service  Manage launchd service

Arguments:
  [PATH]  Target directory [default: .]

Options:
  -r, --recursive    Recursive deletion
  -y, --yes          Skip confirmation
  -n, --dry-run      Scan only, don't delete
  -q, --quiet        Don't list each file
  -e, --exclude      Exclude patterns
      --force        Allow deleting git-tracked files
      --serve        Daemon mode
      --stats        Show timing
  -h, --help         Print help
  -V, --version      Print version
```

## License

MIT
