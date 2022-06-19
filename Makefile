$(VERBOSE).SILENT:
.PHONY: test
default: test
RELEASE_ROOT:target/release

test:
	cargo test --workspace

lint:
	cargo clippy --workspace
	nix-shell -p lua51Packages.luacheck --command 'luacheck lua/xbase && exit 0 || exit 1'

watch:
	RUST_LOG="trace" cargo watch -x 'build' -x 'run -p xbase-daemon' -w 'sourcekit' -w 'daemon' -w 'proto' -w 'lualib' -c

clean:
	cargo clean
	rm -rf bin;
	rm -rf lua/libxbase.so

install: clean
	mkdir bin
	cargo build --release
	mv target/release/xbase-daemon           ./bin/xbase-daemon
	mv target/release/xbase-sourcekit-helper ./bin/xbase-sourcekit-helper
	mv target/release/libxbase_lualib.dylib  ./lua/libxbase.so
	cargo clean # NOTE: 3.2 GB must be cleaned up
	echo "DONE"

install_debug: clean
	mkdir bin
	cargo build
	ln -sf ../target/debug/xbase-daemon            ./bin/xbase-daemon
	ln -sf ../target/debug/xbase-sourcekit-helper  ./bin/xbase-sourcekit-helper
	ln -sf ../target/debug/libxbase_lualib.dylib   ./lua/libxbase.so
	echo "DONE"
