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
	RUST_LOG="debug" cargo watch -x 'build -p xbase-sourcekit-helper -p xbase-editor-lib' -x 'run -p xbase' -w 'sourcekit' -w 'daemon' -w 'proto' -w 'editor' -c

clean:
	rm -rf bin;
	rm -rf lua/xbase_editor_lib.so

install: clean
	killall xbase xbase-sourcekit-helper || true
	mkdir bin
	cargo build --release
	mv target/release/xbase                        ./bin/xbase
	mv target/release/xbase-sourcekit-helper       ./bin/xbase-sourcekit-helper
	mv ../target/debug/libxbase_editor_lib.dylib   ./lua/xbase_editor_lib.so
	echo "DONE"

install_debug: clean
	mkdir bin
	cargo build
	ln -sf ../target/debug/xbase                       ./bin/xbase
	ln -sf ../target/debug/xbase-sourcekit-helper      ./bin/xbase-sourcekit-helper
	ln -sf ../target/debug/libxbase_editor_lib.dylib   ./lua/xbase_editor_lib.so
	echo "DONE"

free_space:
	cargo clean
