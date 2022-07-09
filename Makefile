$(VERBOSE).SILENT:
.PHONY: test
default: test

DEBUG_ROOT=target/release
RELEASE_ROOT=target/release
XBASE_LOCAL_ROOT=~/.local/share/xbase
ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

test:
	cargo test --workspace

lint:
	cargo clippy --workspace
	nix-shell -p lua51Packages.luacheck --command 'luacheck lua/xbase && exit 0 || exit 1'

watch:
	cargo watch -x 'build -p xbase-sourcekit-helper' -x 'run xbase' -w 'crates' -w 'src' -c

clean:
	rm -rf $(XBASE_LOCAL_ROOT) bin

install: clean
	killall xbase xbase-sourcekit-helper || true
	mkdir $(XBASE_LOCAL_ROOT)
	cargo build -p xbase -p xbase-sourcekit-helper --release
	mv target/release/xbase                        $(XBASE_LOCAL_ROOT)/xbase
	mv target/release/xbase-sourcekit-helper       $(XBASE_LOCAL_ROOT)/xbase-sourcekit-helper
	echo "DONE"

install_debug: clean
	killall xbase xbase-sourcekit-helper || true
	mkdir $(XBASE_LOCAL_ROOT)
	cargo build -p xbase -p xbase-sourcekit-helper
	ln -sf $(ROOT_DIR)/target/debug/xbase                       $(XBASE_LOCAL_ROOT)/xbase
	ln -sf $(ROOT_DIR)/target/debug/xbase-sourcekit-helper      $(XBASE_LOCAL_ROOT)/xbase-sourcekit-helper
	echo "DONE"

free_space:
	cargo clean
