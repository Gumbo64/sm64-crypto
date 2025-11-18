# Main Makefile
ROM_FILE = baserom.us.z64

SM64PC = sm64-port
RELEASE = target/release

PROD = prod
WASM = WASM
WASM_XOR = WASM_XOR

WEB_PROD = frontend/src/scripts/sm64/pkg

RESET  = \033[0m
RED    = \033[31m
GREEN  = \033[32m
YELLOW = \033[33m
BLUE   = \033[34m

# Define the default target that will build both web and CLI versions
all: cli-build web-build


web-build:
	wasm-pack build ./browser-wasm --dev --weak-refs --reference-types -t bundler -d pkg

cli-build:
	@echo "$(GREEN)\nBUILDING CLI VERSION$(RESET)"
	cd cli && cargo build --release

	mkdir -p $(PROD)
	cp $(RELEASE)/main $(PROD)/main

	mkdir -p $(PROD)2
	cp $(RELEASE)/main $(PROD)2/main

clean:
	@echo "Cleaning up..."
	$(MAKE) -C $(SM64PC) clean

.PHONY: all cli-build clean
