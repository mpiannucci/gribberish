# Minimal makefile for Sphinx documentation

SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = source
BUILDDIR      = _build

.PHONY: help clean html rustdoc

help:
	@echo "Please use 'make <target>' where <target> is one of"
	@echo "  html       to make standalone HTML files (with integrated Rust docs)"
	@echo "  rustdoc    to build only the Rust documentation"
	@echo "  clean      to remove build artifacts"

clean:
	rm -rf $(BUILDDIR)
	cd .. && cargo clean --doc

rustdoc:
	cd .. && cargo doc --no-deps --workspace

html: rustdoc
	$(SPHINXBUILD) -M html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS)
	@echo "Creating Rust documentation directory..."
	mkdir -p $(BUILDDIR)/html/api/rust
	@echo "Copying Rust documentation..."
	cp -r ../target/doc/. $(BUILDDIR)/html/api/rust/
	@echo
	@echo "Build finished. The HTML pages are in $(BUILDDIR)/html."
	@echo "View at: file://$(shell pwd)/$(BUILDDIR)/html/index.html"