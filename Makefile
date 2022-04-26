$(VERBOSE).SILENT:
.PHONY: test
default: test

test:
	cargo test --workspace
	nvim --headless --noplugin -u scripts/minimal_init.vim -c "PlenaryBustedDirectory lua/tests/auto/ { minimal_init = './scripts/minimal_init.vim' }"

lint:
	cargo clippy --workspace
	nix-shell -p lua51Packages.luacheck --command 'luacheck lua/xcodebase && exit 0 || exit 1'

docgen:
	nvim --headless --noplugin -u scripts/minimal_init.vim -c "luafile ./scripts/gendocs" -c 'qa'

watchlua:
	cargo watch -x 'build -p libxcodebase' -w 'Cargo.toml' -w 'src' -w 'lua/xcodebase/Cargo.toml' -w 'lua/xcodebase/lib.rs'

watchdaemon:
	RUST_LOG="xcodebase=debug" cargo watch -x 'run --bin xcodebase-daemon --features=daemon' -w 'src' -w 'Cargo.toml'

watchserver:
	RUST_LOG="trace" cargo watch -x 'build --bin xcodebase-server --features=server' -w 'src' -w 'Cargo.toml'
