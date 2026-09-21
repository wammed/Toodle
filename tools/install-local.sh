#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"

echo "Building Toodle (release)..."
(cd "${PROJECT_ROOT}" && cargo build --release)

mkdir -p "${BIN_DIR}"

install -m 755 \
    "${PROJECT_ROOT}/target/release/toodle" \
    "${BIN_DIR}/toodle"

install -m 755 \
    "${PROJECT_ROOT}/target/release/toodle-settings" \
    "${BIN_DIR}/toodle-settings"

echo "Installed Toodle to ${BIN_DIR}"
