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
- Watch mode with `dsk watch`
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

**nix-darwin** (declarative service):
```nix
# flake.nix
{
  inputs.dsk.url = "github:kawayww/ds-store-killer";
}

# darwin-configuration.nix
{ inputs, ... }: {
  imports = [ inputs.dsk.darwinModules.default ];

  services.dsk = {
    enable = true;
    paths = [ "~/Downloads" "~/Projects" ]; # directories to watch recursively (default: ~)
    notify = true;           # optional: macOS notifications
    # force = true;          # optional: delete git-tracked files (DANGER)
    # exclude = [ ".git" ];  # optional: exclude patterns
  };
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
dsk kill                      # current dir, asks before deleting
dsk kill -y                   # just delete, don't ask
dsk kill -n                   # dry-run, scan only
dsk kill -r ~/Projects        # recursive
dsk kill -r --force ~/repo    # include git-tracked files
dsk kill -ry --stats .        # recursive, no confirm, show timing
dsk kill -ry -e node_modules  # exclude pattern

dsk watch ~/Desktop           # watch and auto-delete
dsk watch . -e .git           # watch with exclusions
```

## Git Safety

Deleting git-tracked `.DS_Store` messes up your commit history. By default, `dsk` skips them.

| Mode | Behavior |
|------|----------|
| Default | Skips git-tracked files |
| `--force` | Includes them (still asks for confirmation) |

```bash
dsk kill -r ~/my-repo           # skips tracked files
dsk kill -r --force ~/my-repo   # includes them, still confirms
dsk kill -ry --force ~/my-repo  # includes them, no confirmation
```

Why would `.DS_Store` be tracked?
- Someone forgot `.gitignore`
- Force-added with `git add -f`
- `.gitignore` was added after the fact

> **Note**: Git safety requires `git` to be installed. If git is not found, `dsk` will warn and proceed without the safety check.

## Cache

Non-recursive scans are cached in `$TMPDIR/dsk-cache/`. Auto-invalidates when directory changes.

Recursive mode doesn't cache — can't reliably detect subdirectory changes, so we rescan every time.

## Watch Mode

Watch directories and auto-delete `.DS_Store` files instantly.

**Behavior:**
1. **Recursive**: Monitors all subdirectories automatically (no `-r` flag needed).
2. **Initial Scan**: Performs a full recursive scan and cleanup on startup.
3. **Real-time**: Uses macOS FSEvents to monitor changes efficiently.
4. **Non-interactive**: Automatically deletes files without asking.
5. **Git Safety**:
    - **Default**: Skips git-tracked `.DS_Store` files (logs a warning).
    - **`--force`**: Auto-deletes **ALL** `.DS_Store` files, including git-tracked ones.

> **Note**: Unlike `kill` mode where `--force` just *allows* deletion (with confirmation), in `watch` mode `--force` means **aggressive auto-deletion** of tracked `.DS_Store` files.

```bash
dsk watch                # current dir
dsk watch ~/Desktop      # specific dir
dsk watch . -e .git      # with exclusions
dsk watch --notify       # send macOS notification on delete
dsk watch --force        # DANGER: auto-delete git-tracked .DS_Store files too
```

Runs in foreground, Ctrl+C to stop.

## launchd Service

For background monitoring that survives reboots:

```bash
dsk service install                       # watch ~ by default
dsk service install ~/Desktop ~/Projects  # multi-path
dsk service install --notify              # enable notifications
dsk service install --force               # (DANGER) delete git-tracked files
dsk service install -e Downloads          # exclude patterns
dsk service start
dsk service status
dsk service stop
dsk service uninstall
```

Uses **FSEvents** (via `notify` crate) — watching deep directory trees is efficient and doesn't consume file descriptors per subdirectory (solving `os error 24`).

Logs: `/tmp/dsk.out.log`, `/tmp/dsk.err.log`

## CLI Reference

```
Usage: dsk <COMMAND>

Commands:
  kill     Kill .DS_Store files
  watch    Watch directory and auto-delete
  service  Manage launchd service
  help     Print help

dsk kill [OPTIONS] [PATH]
  -r, --recursive    Recursive deletion
  -e, --exclude      Exclude patterns
  -y, --yes          Skip confirmation
  -n, --dry-run      Scan only, don't delete
  -q, --quiet        Don't list each file
      --force        Allow deleting git-tracked .DS_Store files
      --stats        Show timing

dsk watch [PATH]
  -e, --exclude      Exclude patterns
      --notify       Send macOS notification on delete
      --force        (DANGER) Auto-delete git-tracked .DS_Store files

dsk service install [PATHS...] [OPTIONS]
  -e, --exclude      Exclude patterns (persisted in plist)
      --notify       Enable macOS notifications
      --force        (DANGER) Delete git-tracked .DS_Store files

dsk service <uninstall|start|stop|status>
```

## License

MIT
