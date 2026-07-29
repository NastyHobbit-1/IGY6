#!/usr/bin/env bash
set -euo pipefail

# IGY6 Easy Installer
# Builds the compiled 'igy6' executable (Rust CLI) and installs it for easy use.
# Running 'igy6' (bare) will start the full Docker stack (detached) and open your browser to the UI.
# Requires: Rust (cargo), Docker, Docker Compose.
# DIFF-268: also installs local media extraction tools used by the worker/host path.

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

install_media_tools() {
  echo "Installing local media extraction tools (PDF/OCR/ffmpeg/whisper)..."
  if command -v apt-get >/dev/null 2>&1; then
    if command -v sudo >/dev/null 2>&1; then
      sudo apt-get update
      sudo apt-get install -y --no-install-recommends \
        poppler-utils tesseract-ocr tesseract-ocr-eng ffmpeg python3 python3-pip || true
    else
      apt-get update
      apt-get install -y --no-install-recommends \
        poppler-utils tesseract-ocr tesseract-ocr-eng ffmpeg python3 python3-pip || true
    fi
    if command -v pip3 >/dev/null 2>&1; then
      pip3 install --user openai-whisper || true
    fi
  elif command -v brew >/dev/null 2>&1; then
    brew install poppler tesseract ffmpeg || true
    if command -v pip3 >/dev/null 2>&1; then
      pip3 install --user openai-whisper || true
    fi
  else
    echo "NOTE: Could not auto-install media tools (no apt-get/brew)."
    echo "Worker Docker image still installs them on rebuild."
  fi
  echo "Media tool check:"
  command -v pdftotext >/dev/null 2>&1 && echo "  pdftotext: ok" || echo "  pdftotext: missing (worker image provides it)"
  command -v tesseract >/dev/null 2>&1 && echo "  tesseract: ok" || echo "  tesseract: missing (worker image provides it)"
  command -v ffmpeg >/dev/null 2>&1 && echo "  ffmpeg: ok" || echo "  ffmpeg: missing (worker image provides it)"
  command -v whisper >/dev/null 2>&1 && echo "  whisper: ok" || echo "  whisper: missing (worker image provides it)"
}

install_media_tools

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

# Persist repo location for global installs (parity with install.ps1)
for rc_file in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.bash_profile"; do
  if [ -f "$rc_file" ]; then
    if ! grep -q 'export IGY6_REPO=' "$rc_file" 2>/dev/null; then
      echo "export IGY6_REPO=\"$REPO_ROOT\"" >> "$rc_file"
      echo "Set IGY6_REPO in $rc_file"
    fi
  fi
done
export IGY6_REPO="$REPO_ROOT"

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
echo "Docker must be running. Rebuild worker after this install so media tools are in the image:"
echo "  docker compose -f infra/docker-compose.yml build worker"
echo "First run will bootstrap .env with password 'ThatDog123' etc."
echo "igy6 auto-picks WEB_PORT/APP_PORT if 3000/8000 are busy and verifies the IGY6 UI before opening the browser."
echo ""
echo "Optional local LLM (Ollama):"
echo "  scripts/ollama-local-setup.sh --check"
echo "  scripts/ollama-local-setup.sh --install --yes   # Linux only"
echo "If Ollama is already running, igy6 start auto-enables LLM_PROVIDER=ollama."
echo ""
echo "Full working guide: docs/WORKING.md"
