#!/usr/bin/env bash
# Regenerates the screenshots referenced in docs/layout.md from the Rust snippets in that
# same file. See examples/gen_doc_screenshots.rs for how snippets are matched to images.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo run --quiet --example gen_doc_screenshots
cargo run --quiet --example doc_screenshots --features headless
