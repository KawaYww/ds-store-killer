# dsk

> Kill those pesky `.DS_Store` files on macOS
> ?
> F**king .DS_Store ! ! ! ! !

**macOS users only** — because only Apple sprinkles these mysterious little files everywhere in your projects. Linux/Windows folks, congrats on not having to deal with this nonsense.

A fast, minimal CLI tool to eliminate the annoying `.DS_Store` files that macOS sh*ts everywhere.

## Features

- **Interactive confirmation** - Review files before deletion (use `-y` to skip)
- **Dry-run mode** - Preview what would be deleted without removing anything
- **Single/Recursive deletion** - Clean up one directory or an entire tree
- **Daemon mode** - Watch directories and auto-delete new `.DS_Store` files
- **launchd integration** - Run as a background service on boot
- **Exclude patterns** - Skip directories like `node_modules` or `.git`
- **Parallel traversal** - Uses `jwalk` (rayon-based) for blazing fast scanning

## Installation

### From crates.io

```bash
cargo install ds-store-killer
```

### For Nix users

> Nix users might be the only ones installing this on non-macOS systems — but remember, this tool is pretty useless on Linux.


```bash
# Run directly
nix run github:kawayww/ds-store-killer

# Add to flake inputs
{
  inputs.dsk.url = "github:kawayww/ds-store-killer";
}

# Use in darwin-configuration
environment.systemPackages = [ inputs.dsk.packages.${system}.default ];
```

### Build from source

```bash
git clone https://github.com/kawayww/ds-store-killer
cd ds-store-killer
cargo build --release
cp target/release/dsk ~/.local/bin/
```

## Usage

```bash
# Delete .DS_Store in current directory (with confirmation)
dsk

# Delete without asking
dsk -y

# Preview what would be deleted (dry-run)
dsk -n

# Recursive deletion
dsk -ry ~/Projects

# Exclude specific directories
dsk -ry . -e node_modules -e .git

# Quiet mode (no file listing)
dsk -ryq .

# Show execution statistics
dsk -ry --stats ~/Projects
```

## Daemon Mode (`--serve`)

Watch a directory and automatically delete any new `.DS_Store` files as they're created:

```bash
# Watch current directory
dsk --serve

# Watch a specific directory
dsk --serve ~/Desktop

# Watch with exclusions
dsk --serve ~/Projects -e node_modules -e .git
```

The daemon runs in the foreground and monitors the directory **recursively**. Press `Ctrl+C` to stop.

> **Note**: Currently `--serve` only accepts a single path. For watching multiple directories, use the launchd service instead.

## launchd Service

For persistent background monitoring that survives reboots, use the launchd service:

### Service Commands

| Command | Description |
|---------|-------------|
| `dsk service install [PATHS...]` | Create launchd plist with specified watch paths |
| `dsk service uninstall` | Remove plist and stop service |
| `dsk service start` | Load and start the service |
| `dsk service stop` | Unload and stop the service |
| `dsk service status` | Show service status and log locations |

### Examples

```bash
# Install service to watch home directory (default)
dsk service install

# Install service to watch specific directories
dsk service install ~/Desktop ~/Downloads

# Install service to watch multiple project directories
dsk service install ~ ~/Projects ~/Documents

# Start the service after installation
dsk service start

# Check if it's running
dsk service status

# View logs
tail -f /tmp/dsk.out.log

# Stop and remove
dsk service stop
dsk service uninstall
```

### Multi-Path Monitoring

`dsk service install` accepts **multiple paths** as arguments. All paths are monitored by a single service process using the kqueue event mechanism:

- **Performance**: kqueue is event-based, not polling. Watching 10 directories has the same CPU overhead as watching 1.
- **Single process**: One background process handles all paths, minimal memory footprint.
- **Recursive**: Each path is monitored recursively, including all subdirectories.

### How It Works

1. `install` creates a launchd plist at `~/Library/LaunchAgents/com.dsk.guard.plist`
2. `start` loads the plist, launching `dsk --serve` with all configured paths
3. The service runs with `KeepAlive=true`, restarting automatically if it crashes
4. `RunAtLoad=true` ensures it starts on every login

### Logs

- **stdout**: `/tmp/dsk.out.log` — normal operation logs
- **stderr**: `/tmp/dsk.err.log` — errors and warnings

## CLI Reference

```
Usage: dsk [OPTIONS] [PATH] [COMMAND]

Commands:
  service  Manage launchd service

Arguments:
  [PATH]  Target directory [default: .]

Options:
  -r, --recursive          Recursive deletion
      --serve              Daemon mode: watch and auto-delete
  -e, --exclude <PATTERN>  Exclude patterns
  -y, --yes                Skip confirmation, delete directly
  -n, --dry-run            Dry-run: scan only, don't delete
  -q, --quiet              Quiet mode: suppress file listing
      --stats              Show execution statistics
  -h, --help               Print help
  -V, --version            Print version
```

### Service Subcommands

```
Usage: dsk service <COMMAND>

Commands:
  install    Install launchd plist [PATHS...]
  uninstall  Uninstall launchd plist
  start      Start service
  stop       Stop service
  status     Show status
```

## Performance

`dsk` uses [jwalk](https://crates.io/crates/jwalk) for directory traversal. jwalk leverages rayon internally for parallel processing, making it 2-4x faster than `walkdir` + `rayon` for deeply nested directories.

## License

[MIT](LICENSE)

