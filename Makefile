# Main Makefile
ROM_FILE = baserom.us.z64

SM64PC = sm64-port
RELEASE = target/release

PROD = prod
WEB_PROD = web/pkg

RESET  = \033[0m
RED    = \033[31m
GREEN  = \033[32m
YELLOW = \033[33m
BLUE   = \033[34m

# Define the default target that will build both web and CLI versions
all: cli-build web-build


# CLI Build
cli-build:
	@echo "$(GREEN)\nBUILDING CLI VERSION$(RESET)"
	mkdir -p $(PROD)
	cargo build --release
	cp $(RELEASE)/main $(PROD)/main
	
	@echo "$(BLUE)\nNORMAL$(RESET)"
	$(MAKE) -C $(SM64PC) -j
	cp $(SM64PC)/build/us_pc/sm64.us $(PROD)/sm64.us

	@echo "$(BLUE)\nHEADLESS$(RESET)"
	$(MAKE) -C $(SM64PC) -j HEADLESS_VERSION=1
	cp $(SM64PC)/build/us_pc_headless/sm64.us $(PROD)/sm64_headless.us

	mkdir -p $(PROD)2
	cp $(PROD)/main $(PROD)/sm64.us $(PROD)/sm64_headless.us $(PROD)2/
# Web Build

# Check if Emsdk is installed
EMSDK_PATH := $(shell which emcc)

web-build:
	@echo "$(GREEN)\nBUILDING WEB VERSION$(RESET)"
	mkdir -p $(WEB_PROD); 
	@if [ -z "$(EMSDK_PATH)" ]; then \
		echo "$(YELLOW)Skipping Web Build: Emscripten SDK (emsdk) is not installed.$(RESET)"; \
	else \
		echo "$(BLUE)\nNORMAL$(RESET)"; \
		$(MAKE) -C $(SM64PC) -j TARGET_WEB=1; \
		cp $(SM64PC)/build/us_web/sm64.us.wasm $(WEB_PROD)/sm64.us.wasm; \
		cp $(SM64PC)/build/us_web/sm64.us.js $(WEB_PROD)/sm64.us.js; \
		echo "$(BLUE)\nHEADLESS$(RESET)"; \
		$(MAKE) -C $(SM64PC) -j TARGET_WEB=1 HEADLESS_VERSION=1; \
		cp $(SM64PC)/build/us_web_headless/sm64.us.wasm $(WEB_PROD)/sm64_headless.us.wasm; \
		cp $(SM64PC)/build/us_web_headless/sm64.us.js $(WEB_PROD)/sm64_headless.us.js; \
	fi

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
