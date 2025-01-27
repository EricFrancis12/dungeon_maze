SANDBOX_CARGO_TOML_PATH := ./crates/sandbox/Cargo.toml
EDITOR_DIR_PATH := ./editor

build_sandbox:
	cargo build --manifest-path $(SANDBOX_CARGO_TOML_PATH)

run_sandbox:
	cargo run --manifest-path $(SANDBOX_CARGO_TOML_PATH)

install_editor:
	cd $(EDITOR_DIR_PATH) && npm i

run_editor:
	cd $(EDITOR_DIR_PATH) && npm run dev
