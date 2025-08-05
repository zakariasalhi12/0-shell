SHELL_BIN=$(HOME)/.push/bin/push
CAT_BIN=$(HOME)/.push/bin/cat
LS_BIN=$(HOME)/.push/bin/ls


install:
	mkdir -p ~/.push/bin
	touch ~/.push/.pushrc
	echo 'export PATH=$$HOME/.push/bin:$$PATH' >> ~/.push/.pushrc
	$(MAKE) clean
	$(MAKE) all

all: $(SHELL_BIN) $(CAT_BIN) $(LS_BIN)

$(SHELL_BIN):
	cargo build --release --manifest-path=shell/Cargo.toml
	cp shell/target/release/shell $(SHELL_BIN)

$(CAT_BIN):
	cargo build --release --manifest-path=cat/Cargo.toml
	cp cat/target/release/cat $(CAT_BIN)

$(LS_BIN):
	cargo build --release --manifest-path=ls/Cargo.toml
	cp ls/target/release/ls $(LS_BIN)

clean: cargo-clean
	rm -f $(HOME)/.push/bin/*

cargo-clean:
	cargo clean --manifest-path=shell/Cargo.toml
	cargo clean --manifest-path=cat/Cargo.toml
	cargo clean --manifest-path=ls/Cargo.toml

run: clean all
	./$(SHELL_BIN)

push: cargo-clean clean
	@read -p "Enter commit message: " msg; \
	git add .; \
	git commit -m "$$msg"; \
	git push
