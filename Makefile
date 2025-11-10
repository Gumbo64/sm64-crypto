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
all: cli-build sm64-build


# CLI Build
cli-build: sm64-build
	@echo "$(GREEN)\nBUILDING CLI VERSION$(RESET)"
	cargo build --release

	mkdir -p $(PROD)
	cp $(RELEASE)/main $(PROD)/main
	cp $(WASM)/sm64.us.wasm $(PROD)/sm64.us.wasm
	cp $(WASM)/sm64_headless.us.wasm $(PROD)/sm64_headless.us

	mkdir -p $(PROD)2
	cp $(RELEASE)/main $(PROD)2/main
	cp $(WASM)/sm64.us.wasm $(PROD)2/sm64.us.wasm
	cp $(WASM)/sm64_headless.us.wasm $(PROD)2/sm64_headless.us


# Check if Emsdk is installed
EMSDK_PATH := $(shell which emcc)

sm64-build: copy-rom
	@echo "$(GREEN)\nBUILDING WEB VERSION$(RESET)"
	mkdir -p $(WEB_PROD); 
	@if [ -z "$(EMSDK_PATH)" ]; then \
		echo "$(YELLOW)Emscripten SDK (emsdk) is not installed, recreating WASM from XOR files $(RESET)"; \
		python3 maketools/xor.py $(WASM_XOR)/sm64.us.wasm.xor $(WASM)/sm64.us.wasm  $(ROM_FILE); \
		python3 maketools/xor.py $(WASM_XOR)/sm64_headless.us.wasm.xor $(WASM)/sm64_headless.us.wasm $(ROM_FILE); \
	else \
		echo "$(BLUE)\nNORMAL$(RESET)"; \
		$(MAKE) -C $(SM64PC) -j TARGET_WEB=1; \
		echo "$(BLUE)\nHEADLESS$(RESET)"; \
		$(MAKE) -C $(SM64PC) -j TARGET_WEB=1 HEADLESS_VERSION=1; \
		cp $(SM64PC)/build/us_web/sm64.us.js $(WEB_PROD)/sm64.us.js; \
		cp $(SM64PC)/build/us_web/sm64.us.wasm $(WASM)/sm64.us.wasm; \
		cp $(SM64PC)/build/us_web_headless/sm64.us.wasm $(WASM)/sm64_headless.us.wasm; \
		python3 maketools/xor.py $(WASM)/sm64.us.wasm $(WASM_XOR)/sm64.us.wasm.xor $(ROM_FILE); \
		python3 maketools/xor.py $(WASM)/sm64_headless.us.wasm $(WASM_XOR)/sm64_headless.us.wasm.xor $(ROM_FILE); \
	fi
# 	wasm-pack build browser-wasm --release -t bundler -d $(WEB_PROD)

# Define a target to copy the ROM file into the SM64PC
copy-rom:
	@echo "Copying $(ROM_FILE) to $(SM64PC)..."
	@if [ ! -f $(ROM_FILE) ]; then \
		echo -e "\033[0;31mA Mario 64 ROM is required with the name $(ROM_FILE)\033[0m"; \
		exit 1; \
	fi
	
	cp $(ROM_FILE) $(SM64PC)/$(ROM_FILE)

clean:
	@echo "Cleaning up..."
	$(MAKE) -C $(SM64PC) clean

.PHONY: all web-build cli-build copy-rom clean
