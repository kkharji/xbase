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

