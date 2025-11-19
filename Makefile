# Main Makefile
RELEASE = target/release
PROD = prod

RESET  = \033[0m
RED    = \033[31m
GREEN  = \033[32m
YELLOW = \033[33m
BLUE   = \033[34m

# Define the default target that will build both web and CLI versions
all: cli-build web-build


web-build:
	cargo install wasm-pack wasm-bindgen-cli
	cd frontend && npm run build:wasm:release

cli-build:
	@echo "$(GREEN)\nBUILDING CLI VERSION$(RESET)"
	cd cli && cargo build --release

	mkdir -p $(PROD)
	cp $(RELEASE)/main $(PROD)/main

clean:
	@echo "Cleaning up..."
	$(MAKE) -C $(SM64PC) clean

.PHONY: all cli-build clean
