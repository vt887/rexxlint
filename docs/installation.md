# Installation

## From pre-built binaries (recommended)

Download the archive for your platform from the
[latest GitHub Release](https://github.com/vt887/rexxlint/releases/latest).

### Linux x86-64

```bash
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/rexxlint-linux-x86_64.tar.gz
tar -xzf rexxlint-linux-x86_64.tar.gz
sudo install -m 755 rexxlint /usr/local/bin/
```

### Linux AArch64 (ARM64)

```bash
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/rexxlint-linux-aarch64.tar.gz
tar -xzf rexxlint-linux-aarch64.tar.gz
sudo install -m 755 rexxlint /usr/local/bin/
```

### macOS (Intel)

```bash
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/rexxlint-macos-x86_64.tar.gz
tar -xzf rexxlint-macos-x86_64.tar.gz
sudo install -m 755 rexxlint /usr/local/bin/
```

### macOS (Apple Silicon)

```bash
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/rexxlint-macos-aarch64.tar.gz
tar -xzf rexxlint-macos-aarch64.tar.gz
sudo install -m 755 rexxlint /usr/local/bin/
```

### Windows x86-64

Download `rexxlint-windows-x86_64.zip` from the release page, extract it, and
add the folder containing `rexxlint.exe` to your `PATH`.

PowerShell:

```powershell
Expand-Archive rexxlint-windows-x86_64.zip -DestinationPath "$env:LOCALAPPDATA\rexxlint"
[Environment]::SetEnvironmentVariable(
  "PATH",
  "$env:PATH;$env:LOCALAPPDATA\rexxlint",
  "User"
)
```

### Verify the checksum

```bash
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/SHA256SUMS
sha256sum --check --ignore-missing SHA256SUMS
```

## From source (cargo install)

Requires Rust stable (≥ 1.85).

```bash
cargo install --git https://github.com/vt887/rexxlint rexx-cli --bin rexxlint
```

Or, if you have cloned the repository:

```bash
cargo install --path crates/rexx-cli
```

## Verify installation

```bash
rexxlint --version
# rexxlint: 0.2.0-alpha 2026-05-16 (64 bit)
```

## Package managers (future)

> These are placeholders; packages are not yet published.

### Homebrew (macOS / Linux)

```bash
# Not yet available
brew install vt887/tap/rexxlint
```

### Scoop (Windows)

```powershell
# Not yet available
scoop bucket add rexxlint https://github.com/vt887/scoop-rexxlint
scoop install rexxlint
```

## PATH configuration

If you installed to a non-standard location, add it to your `PATH`:

**bash / zsh** (`~/.bashrc` or `~/.zshrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**fish** (`~/.config/fish/config.fish`):

```fish
set -gx PATH $HOME/.local/bin $PATH
```

**PowerShell** (`$PROFILE`):

```powershell
$env:PATH += ";$env:LOCALAPPDATA\rexxlint"
```
