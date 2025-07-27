SHELL_BIN=bin/0shell
CAT_BIN=bin/cat

all: $(SHELL_BIN) $(CAT_BIN)

$(SHELL_BIN):
	cargo build --release --manifest-path=shell/Cargo.toml
	cp shell/target/release/shell $(SHELL_BIN)

$(CAT_BIN):
	cargo build --release --manifest-path=cat/Cargo.toml
	cp cat/target/release/cat $(CAT_BIN)

clean:
	rm -f bin/*

run:clean  all
	./$(SHELL_BIN)
