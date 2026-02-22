BINARY_NAME := claude-hook
INSTALL_DIR := $(HOME)/.claude/hooks
OLD_SCRIPTS := bash-permission.sh read-permission.sh web-permission.sh

.PHONY: all test build install deploy clean uninstall

all: test build

test:
	cargo test

build:
	cargo build --release

install: build
	@mkdir -p $(INSTALL_DIR)
	cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installed $(BINARY_NAME) to $(INSTALL_DIR)"

deploy: install
	@echo "Cleaning up old bash scripts..."
	@for f in $(OLD_SCRIPTS); do \
		rm -f $(INSTALL_DIR)/$$f && echo "  Removed $$f"; \
	done
	@echo ""
	@echo "Deploy complete."
	@echo "See README.md for settings.json configuration."

clean:
	cargo clean

uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Removed $(BINARY_NAME) from $(INSTALL_DIR)"
