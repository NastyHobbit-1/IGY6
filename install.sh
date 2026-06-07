#!/usr/bin/env bash
set -euo pipefail

# IGY6 Easy Installer
# Builds the compiled 'igy6' executable (Rust CLI) and installs it for easy use.
# Running 'igy6' (bare) will start the full Docker stack (detached) and open your browser to the UI.
# Requires: Rust (cargo), Docker, Docker Compose.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR"

echo "=== IGY6 Installer ==="
echo "Repo: $REPO_ROOT"

# Check prerequisites
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo (Rust) not found."
  echo "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker not found. Please install Docker and Docker Compose."
  exit 1
fi

# Build the executable
echo "Building igy6 CLI (this may take a minute on first build)..."
cd "$REPO_ROOT"
cargo build -p igy6-cli --release

BINARY="$REPO_ROOT/target/release/igy6"
if [ ! -f "$BINARY" ]; then
  echo "ERROR: Build failed, binary not found at $BINARY"
  exit 1
fi

# Install to user bin
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"
cp -f "$BINARY" "$INSTALL_DIR/igy6"
chmod +x "$INSTALL_DIR/igy6"

echo "Installed igy6 to $INSTALL_DIR/igy6"

# Update shell PATH (idempotent)
for rc_file in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.bash_profile"; do
  if [ -f "$rc_file" ]; then
    if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$rc_file" 2>/dev/null; then
      echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rc_file"
      echo "Updated $rc_file with PATH"
    fi
  fi
done

echo ""
echo "=== Installation complete ==="
echo "To use immediately:"
echo "  source ~/.bashrc   # or your shell rc"
echo "  igy6               # starts the stack (detached) and opens browser to UI"
echo ""
echo "Other commands:"
echo "  igy6 --help"
echo "  igy6 start"
echo "  igy6 stop"
echo "  igy6 health"
echo ""
echo "Note: Keep this repo directory ($REPO_ROOT) - the binary auto-finds it."
echo "Docker must be running."
echo "First run will bootstrap .env with password 'ThatDog123' etc."
