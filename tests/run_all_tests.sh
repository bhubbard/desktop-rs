#!/usr/bin/env bash
set -euo pipefail

echo "======================================================="
echo "🧪 Running GitHub Desktop Compatibility & Verification"
echo "======================================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

echo "1. Running Workspace Unit & Integration Tests..."
cargo test --workspace

echo ""
echo "2. Running GitHub Desktop Diff/Status Parser Unit Compat Tests..."
cargo test -p desktop-core --test github_desktop_unit_compat -- --nocapture

echo ""
echo "3. Running GitHub Desktop Official Fixtures Compat Tests..."
cargo test -p desktop-git --test github_desktop_fixtures_compat -- --nocapture

echo ""
echo "4. Running End-to-End CLI Porcelain Tests against GitHub Desktop Fixtures..."
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Setup repo-with-changes fixture
cp -r "$REPO_ROOT/tests/fixtures/repo-with-changes" "$TMP_DIR/repo-with-changes"
mv "$TMP_DIR/repo-with-changes/_git" "$TMP_DIR/repo-with-changes/.git"

echo "  -> Verifying 'desktop status' on repo-with-changes..."
cargo run --bin desktop -- status -C "$TMP_DIR/repo-with-changes"

echo "  -> Verifying 'desktop diff' on repo-with-changes..."
cargo run --bin desktop -- diff -C "$TMP_DIR/repo-with-changes"

# Setup test-repo fixture
cp -r "$REPO_ROOT/tests/fixtures/test-repo" "$TMP_DIR/test-repo"
mv "$TMP_DIR/test-repo/_git" "$TMP_DIR/test-repo/.git"

echo "  -> Verifying 'desktop log' on test-repo..."
cargo run --bin desktop -- log -n 5 -C "$TMP_DIR/test-repo"

echo ""
echo "======================================================="
echo "✅ All GitHub Desktop compatibility tests passed!"
echo "======================================================="
