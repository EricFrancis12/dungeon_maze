GAME_CARGO_TOML_PATH := ./crates/game/Cargo.toml
SANDBOX_CARGO_TOML_PATH := ./crates/sandbox/Cargo.toml
EDITOR_DIR_PATH := ./editor

# game
build:
	cargo build --manifest-path $(GAME_CARGO_TOML_PATH)

run:
	cargo run --manifest-path $(GAME_CARGO_TOML_PATH)

build_release:
	cargo build --manifest-path $(GAME_CARGO_TOML_PATH) --release

run_release:
	cargo run --manifest-path $(GAME_CARGO_TOML_PATH) --release

# sandbox
run_sandbox:
	cargo run --manifest-path $(SANDBOX_CARGO_TOML_PATH)

build_sandbox:
	cargo build --manifest-path $(SANDBOX_CARGO_TOML_PATH)

# editor
install_editor:
	cd $(EDITOR_DIR_PATH) && npm i

run_editor:
	cd $(EDITOR_DIR_PATH) && npm run dev
