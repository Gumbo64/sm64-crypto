# Main Makefile
ROM_FILE = baserom.us.z64

SM64PC = sm64-port
RELEASE = target/release
PROD = prod

# Define the target that will copy the ROM file and call the Makefile in SM64PC
all: copy-rom
	@echo "Calling Makefile in $(SM64PC)..."


# 	WEB BUILDING
	$(MAKE) -C $(SM64PC) -j TARGET_WEB=1
	$(MAKE) -C $(SM64PC) -j TARGET_WEB=1 HEADLESS_VERSION=1
	cp $(SM64PC)/build/us_web_headless/sm64.us.wasm frontend/pkg/sm64_headless.us.wasm
	cp $(SM64PC)/build/us_web_headless/sm64.us.js frontend/pkg/sm64_headless.us.js
	cp $(SM64PC)/build/us_web/sm64.us.wasm frontend/pkg/sm64.us.wasm
	cp $(SM64PC)/build/us_web/sm64.us.js frontend/pkg/sm64.us.js

#   CLI BUILDING
	$(MAKE) -C $(SM64PC) -j
	$(MAKE) -C $(SM64PC) -j HEADLESS_VERSION=1
	cargo build --release
	mkdir -p $(PROD)

	cp $(RELEASE)/main $(PROD)/main
	cp $(SM64PC)/build/us_pc_headless/sm64.us $(PROD)/sm64_headless.us
	cp $(SM64PC)/build/us_pc/sm64.us $(PROD)/sm64.us

	
	mkdir -p $(PROD)2
	cp -r $(PROD)/* $(PROD)2


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

.PHONY: all copy-rom clean
