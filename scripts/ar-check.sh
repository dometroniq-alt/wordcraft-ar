#!/usr/bin/env bash
# The fork's quick gate: the Arabic tests, the UI kit, the icons, the self-test and clippy on the
# two crates the fork touches — a couple of minutes, not the whole `cargo xtask ci`.
# Usage: scripts/ar-check.sh            (add --sheet to also render the thumbnail/icon sheets
#        into target/ar-sheets/ for a look)
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}
cargo fmt --check
cargo clippy -p wordcraft-engine -p wordcraft-ui-egui --all-targets -- -D warnings
cargo test -p wordcraft-engine -- service numeral arabic
cargo test -p wordcraft-ui-egui -- ui_kit visual_help icons dev_tools rtl i18n::tests keytips
if [[ "${1:-}" == "--sheet" ]]; then
  mkdir -p target/ar-sheets
  cargo run -q -p wordcraft-ui-egui --example kit_sheet -- thumbs target/ar-sheets/thumbnails.png
  cargo run -q -p wordcraft-ui-egui --example kit_sheet -- icons target/ar-sheets/icons.png
  echo "sheets in target/ar-sheets/"
fi
echo "ar-check: ok"
