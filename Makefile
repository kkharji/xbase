$(VERBOSE).SILENT:
.PHONY: test
default: test
RELEASE_ROOT:target/release

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
	RUST_LOG="xbase=trace" cargo watch -x 'run --bin xbase-daemon --features=daemon' -w 'src' -w 'Cargo.toml' -c

watchserver:
	RUST_LOG="trace" cargo watch -x 'build -p xbase-sourcekit-helper' -w 'crates/sourcekit/' -c

clean:
	rm -rf bin;
	rm -rf lua/libxbase.so

install: clean
	mkdir bin
	cargo build --release -p xbase-sourcekit-helper
	cargo build --release --bin xbase-daemon --features=daemon
	cargo build --release -p libxbase
	mv target/release/xbase-daemon       ./bin/xbase-daemon
	ln -sf ../target/debug/xbase-sourcekit-helper ./bin/xbase-sourcekit-helper
	mv target/release/liblibxbase.dylib  ./lua/libxbase.so
	cargo clean # NOTE: 3.2 GB must be cleaned up
	echo "DONE"

install_debug: clean
	mkdir bin
	cargo build -p xbase-sourcekit-helper
	cargo build --bin xbase-daemon --features=daemon
	cargo build -p libxbase
	ln -sf ../target/debug/xbase-daemon  ./bin/xbase-daemon
	ln -sf ../target/debug/xbase-sourcekit-helper ./bin/xbase-sourcekit-helper
	ln -sf ../target/debug/liblibxbase.dylib ./lua/libxbase.so
	echo "DONE"
