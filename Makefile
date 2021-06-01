# tested with make 4.2.1

# required binaries
CARGO := cargo
RUSTC := rustc
CC := gcc
LDD := ldd
BUILDAH := buildah
GIT := git
JQ := jq
CURL := curl
UNZIP := unzip
PODMAN := podman

ARCH = $(shell arch)

GIT_BRANCH := $(shell $(GIT) rev-parse --abbrev-ref HEAD)
GIT_COMMIT := $(shell $(GIT) rev-parse --short HEAD)
GIT_VERSION := $(GIT_BRANCH)/$(GIT_COMMIT)

APP_NAME := $(shell $(CARGO) read-manifest | $(JQ) -r .name)
APP_VERSION := $(shell $(CARGO) read-manifest | $(JQ) -r .version)
APP_REPOSITORY := $(shell $(CARGO) read-manifest | $(JQ) -r .repository)
APP_OWNER := jostho

IMAGE_BINARY_PATH := /usr/local/bin/$(APP_NAME)
IMAGE_META_VERSION_PATH := /usr/local/etc/$(APP_NAME)-release
IMAGE_SHARE_PATH := /usr/local/share
PORT := 8000

LOCAL_META_VERSION_PATH := $(CURDIR)/target/meta.version

TARGET_MUSL := $(ARCH)-unknown-linux-musl

COUNTRY_FLAGS := country-flags
COUNTRY_FLAGS_ARCHIVE_URL := https://github.com/hampusborgos/$(COUNTRY_FLAGS)/archive/main.zip
COUNTRY_FLAGS_LOCAL_ARCHIVE := $(CURDIR)/target/main.zip
COUNTRY_FLAGS_LOCAL_DIR := $(CURDIR)/target/$(COUNTRY_FLAGS)-main

RUSTC_PRINT_TARGET_CMD := $(RUSTC) -Z unstable-options --print target-spec-json
JQ_TARGET_CMD := $(JQ) -r '."llvm-target"'

# github action sets "CI=true"
ifeq ($(CI), true)
IMAGE_PREFIX := ghcr.io/$(APP_OWNER)
IMAGE_VERSION := $(GIT_COMMIT)
else
IMAGE_PREFIX := $(APP_OWNER)
IMAGE_VERSION := v$(APP_VERSION)
endif

check: check-required check-optional

check-required:
	$(CARGO) --version
	$(RUSTC) --version
	$(CC) --version | head -1
	$(LDD) --version | head -1

check-optional:
	$(BUILDAH) --version
	$(GIT) --version
	$(JQ) --version
	$(CURL) --version | head -1
	$(UNZIP) -h | head -1
	$(PODMAN) --version

clean:
	$(CARGO) clean

build:
	$(CARGO) build --release

build-static:
	$(CARGO) build --release --target $(TARGET_MUSL)

check-target-dir:
	test -d $(CURDIR)/target

prep-version-file: check-target-dir
	echo "$(APP_NAME) $(APP_VERSION) $(LLVM_TARGET)" > $(LOCAL_META_VERSION_PATH)
	$(MAKE) -s check-required >> $(LOCAL_META_VERSION_PATH)

get-flags: check-target-dir
	test -f $(COUNTRY_FLAGS_LOCAL_ARCHIVE) || $(CURL) -m 60 -L -o $(COUNTRY_FLAGS_LOCAL_ARCHIVE) $(COUNTRY_FLAGS_ARCHIVE_URL)
	rm -rf $(COUNTRY_FLAGS_LOCAL_DIR) && $(UNZIP) -q $(COUNTRY_FLAGS_LOCAL_ARCHIVE) -d $(CURDIR)/target/

# target for Containerfile
build-prep: LLVM_TARGET = $(shell $(RUSTC_PRINT_TARGET_CMD) | $(JQ_TARGET_CMD))
build-prep: build prep-version-file get-flags

build-image-default: BASE_IMAGE = debian
build-image-default:
	$(BUILDAH) bud \
		--tag $(IMAGE_NAME) \
		--label app-name=$(APP_NAME) \
		--label app-version=$(APP_VERSION) \
		--label app-git-version=$(GIT_VERSION) \
		--label app-arch=$(ARCH) \
		--label app-base-image=$(BASE_IMAGE) \
		--label org.opencontainers.image.source=$(APP_REPOSITORY) \
		-f Containerfile .

build-image-static: BASE_IMAGE = scratch
build-image-static: CONTAINER = $(APP_NAME)-$(BASE_IMAGE)-build-1
build-image-static: LOCAL_BINARY_PATH = $(CURDIR)/target/$(TARGET_MUSL)/release/$(APP_NAME)
build-image-static:
	$(BUILDAH) from --name $(CONTAINER) $(BASE_IMAGE)
	$(BUILDAH) copy $(CONTAINER) $(LOCAL_BINARY_PATH) $(IMAGE_BINARY_PATH)
	$(BUILDAH) copy $(CONTAINER) $(LOCAL_META_VERSION_PATH) $(IMAGE_META_VERSION_PATH)
	$(BUILDAH) copy $(CONTAINER) templates $(IMAGE_SHARE_PATH)/$(APP_NAME)/templates
	$(BUILDAH) copy $(CONTAINER) $(COUNTRY_FLAGS_LOCAL_DIR) $(IMAGE_SHARE_PATH)/$(COUNTRY_FLAGS)
	$(BUILDAH) config \
		--cmd $(IMAGE_BINARY_PATH) \
		--port $(PORT) \
		--env FLOCK_FLAG_DIR=$(IMAGE_SHARE_PATH)/$(COUNTRY_FLAGS) \
		--env FLOCK_TEMPLATE_DIR=$(IMAGE_SHARE_PATH)/$(APP_NAME)/templates \
		-l app-name=$(APP_NAME) \
		-l app-version=$(APP_VERSION) \
		-l app-git-version=$(GIT_VERSION) \
		-l app-arch=$(ARCH) \
		-l app-base-image=$(BASE_IMAGE) \
		-l app-llvm-target=$(LLVM_TARGET) \
		-l org.opencontainers.image.source=$(APP_REPOSITORY) \
		$(CONTAINER)
	$(BUILDAH) commit --rm $(CONTAINER) $(IMAGE_NAME)

verify-image:
	$(BUILDAH) images
	$(PODMAN) run $(IMAGE_NAME) $(IMAGE_BINARY_PATH) --version

push-image:
ifeq ($(CI), true)
	$(BUILDAH) push $(IMAGE_NAME)
endif

image: IMAGE_NAME = $(IMAGE_PREFIX)/$(APP_NAME):$(IMAGE_VERSION)
image: clean build-image-default verify-image

image-static: IMAGE_NAME = $(IMAGE_PREFIX)/$(APP_NAME)-static:$(IMAGE_VERSION)
image-static: LLVM_TARGET = $(shell $(RUSTC_PRINT_TARGET_CMD) --target $(TARGET_MUSL) | $(JQ_TARGET_CMD))
image-static: clean build-static prep-version-file get-flags build-image-static verify-image push-image

.PHONY: check check-required check-optional check-target-dir
.PHONY: clean prep-version-file get-flags
.PHONY: build build-static build-prep
.PHONY: build-image-default build-image-static
.PHONY: verify-image push-image
.PHONY: image image-static
