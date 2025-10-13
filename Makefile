# Main Makefile

SM64ENV = sm64game
SM64PC = $(SM64ENV)/sm64-port

ROM_FILE = baserom.us.z64
RELEASE = rust_crypto/target/release
PROD = prod

# Define the target that will copy the ROM file and call the Makefile in SM64PC
all: copy-rom
	@echo "Calling Makefile in $(SM64PC)..."
	$(MAKE) -C $(SM64PC) -j
	$(MAKE) -C $(SM64PC) -j HEADLESS_VERSION=1

	$(MAKE) -C $(SM64PC) -j TARGET_WEB=1
	$(MAKE) -C $(SM64PC) -j TARGET_WEB=1 HEADLESS_VERSION=1


# 	cp $(SM64PC)/build/us_pc_headless/sm64.us rust_crypto/sm64_headless.us
# 	cp $(SM64PC)/build/us_pc/sm64.us rust_crypto/sm64.us
	cp $(SM64PC)/build/us_web_headless/sm64.us.wasm rust_crypto/sm64_headless.us.wasm
	cp $(SM64PC)/build/us_web/sm64.us.wasm rust_crypto/sm64.us.wasm

	cd rust_crypto && cargo build --release

# 	rm -r prod
# 	rm -r prod2

	mkdir -p prod
	mkdir -p prod2
	cp $(RELEASE)/main $(PROD)/main
	cp $(RELEASE)/evaluate $(PROD)/evaluate
	cp $(RELEASE)/record $(PROD)/record

	cp $(SM64PC)/build/us_pc_headless/sm64.us prod/sm64_headless.us
	cp $(SM64PC)/build/us_pc/sm64.us prod/sm64.us
	cp $(SM64PC)/build/us_web_headless/sm64.us.wasm prod/sm64_headless.us.wasm
	cp $(SM64PC)/build/us_web/sm64.us.wasm prod/sm64.us.wasm


	cp -r prod/* prod2


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
