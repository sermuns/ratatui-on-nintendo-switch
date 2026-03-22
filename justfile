[no-cd]
upload:
	#!/bin/bash
	set -e

	if [[ "$(basename "$(dirname "$PWD")")" != "examples" ]]; then
		echo "Error: must be inside a subdirectory of examples/" >&2
		exit 1
	fi

	TARGET_DIR=$(cargo metadata --format-version=1 --no-deps | jq -r '.target_directory')
	PKG_NAME=$(basename "$PWD")

	cargo nx build --release --package $PKG_NAME
	cargo nx link "$TARGET_DIR/aarch64-nintendo-switch-freestanding/release/$PKG_NAME.nro"
