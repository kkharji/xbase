$(VERBOSE).SILENT:
.PHONY: test
default: test

test:
	cargo test --workspace
	nvim --headless --noplugin -u scripts/minimal_init.vim -c "PlenaryBustedDirectory lua/tests/auto/ { minimal_init = './scripts/minimal_init.vim' }"

lint:
	cargo clippy --workspace
	nix-shell -p lua51Packages.luacheck --command 'luacheck lua/xbase && exit 0 || exit 1'

docgen:
	nvim --headless --noplugin -u scripts/minimal_init.vim -c "luafile ./scripts/gendocs" -c 'qa'

watchlua:
	cargo watch -x 'build -p libxbase' -w 'Cargo.toml' -w 'src' -w 'lua/xbase/Cargo.toml' -w 'lua/xbase/lib.rs' -c

watchdaemon:
	RUST_LOG="xbase=debug" cargo watch -x 'run --bin xbase-daemon --features=daemon' -w 'src' -w 'Cargo.toml' -c

watchserver:
	RUST_LOG="trace" cargo watch -x 'build --bin xbase-server --features=server' -w 'src' -w 'Cargo.toml' -c
