SHELL = /bin/sh
CARGO = cargo
INSTALL = install
INSTALL_PROGRAM = $(INSTALL)

prefix = /usr/local
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin

.PHONY: all
all: elfx86exts

elfx86exts:
	$(CARGO) build
	cp target/debug/elfx86exts .

.PHONY: check
check: elfx86exts
	cargo test

.PHONY: install
install: elfx86exts
	mkdir -p $(DESTDIR)$(bindir)
	$(INSTALL_PROGRAM) elfx86exts $(DESTDIR)$(bindir)/elfx86exts

.PHONY: uninstall
uninstall:
	rm $(bindir)/elfx86exts

.PHONY: clean
clean:
	rm -rf target elfx86exts

