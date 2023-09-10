.DEFAULT_GOAL:=help
SHELL:=/bin/bash
.PHONY: %

# Other PIs should be able to use target armv7-unknown-linux-gnueabihf
rpi01: ## Build for Raspberry Pi 0/1 - requires cross-rs & docker
	cross build -vv --target arm-unknown-linux-gnueabihf

# Build of image separately, so progress and errors can be seen.
rpi01-docker: ## Build docker image for Raspberry Pi 0 or 1
	docker build -f docker/cross-opencv-rpi01.Dockerfile -t cross-opencv-rpi01 .

winpc: ## Build for Windows - requires cross-rs & docker
	cross build --target x86_64-pc-windows-gnu

winpc-docker: ## Build docker image for Windows
	docker build -f docker/cross-opencv-winpc.Dockerfile -t cross-opencv-winpc .

# tput colors
cyan := $(shell tput setaf 6)
reset := $(shell tput sgr0)
#
# Credits for Self documenting Makefile:
# https://www.thapaliya.com/en/writings/well-documented-makefiles/
# https://github.com/awinecki/magicfile/blob/main/Makefile
#
help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make $(cyan)[target ...]$(reset)\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  $(cyan)%-13s$(reset) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
