upload: build
	cargo nx link $CARGO_TARGET_DIR/aarch64-nintendo-switch-freestanding/release/ratatui-on-nx.nro

build:
	cargo nx build --release
