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

ARCH = $(shell arch)

GIT_BRANCH := $(shell $(GIT) rev-parse --abbrev-ref HEAD)
GIT_COMMIT := $(shell $(GIT) rev-parse --short HEAD)
GIT_VERSION := $(GIT_BRANCH)/$(GIT_COMMIT)

APP_NAME := $(shell $(CARGO) read-manifest | $(JQ) -r .name)
APP_VERSION := $(shell $(CARGO) read-manifest | $(JQ) -r .version)

UBI_BASE_IMAGE := registry.access.redhat.com/ubi8-minimal:8.3

IMAGE_BINARY_PATH := /usr/local/bin/$(APP_NAME)
IMAGE_META_VERSION_PATH := /usr/local/etc/$(APP_NAME)-release
IMAGE_SHARE_PATH := /usr/local/share
PORT := 8000

LOCAL_META_VERSION_PATH := $(CURDIR)/target/meta.version

TARGET_MUSL := $(ARCH)-unknown-linux-musl

COUNTRY_FLAGS := country-flags
COUNTRY_FLAGS_ARCHIVE_URL := https://github.com/hjnilsson/$(COUNTRY_FLAGS)/archive/master.zip
COUNTRY_FLAGS_LOCAL_ARCHIVE := $(CURDIR)/target/master.zip
COUNTRY_FLAGS_LOCAL_DIR := $(CURDIR)/target/$(COUNTRY_FLAGS)-master

RUSTC_PRINT_TARGET_CMD := $(RUSTC) -Z unstable-options --print target-spec-json
JQ_TARGET_CMD := $(JQ) -r '."llvm-target"'

check: check-required check-optional

check-required:
	$(CARGO) --version
	$(RUSTC) --version
	$(CC) --version | head -1
	$(LDD) --version | head -1
	$(BUILDAH) --version

check-optional:
	$(GIT) --version
	$(JQ) --version
	$(CURL) --version | head -1
	$(UNZIP) -h | head -1

clean:
	$(CARGO) clean

build:
	$(CARGO) build --release

build-static:
	$(CARGO) build --release --target $(TARGET_MUSL)

prep-version-file:
	mkdir -p $(CURDIR)/target && echo "$(APP_NAME) $(APP_VERSION)" > $(LOCAL_META_VERSION_PATH)
	$(MAKE) -s check-required >> $(LOCAL_META_VERSION_PATH)

get-flags:
	test -f $(COUNTRY_FLAGS_LOCAL_ARCHIVE) || $(CURL) -m 60 -L -o $(COUNTRY_FLAGS_LOCAL_ARCHIVE) $(COUNTRY_FLAGS_ARCHIVE_URL)
	rm -rf $(COUNTRY_FLAGS_LOCAL_DIR) && $(UNZIP) -q $(COUNTRY_FLAGS_LOCAL_ARCHIVE) -d $(CURDIR)/target/

build-image-default: BASE_IMAGE_TYPE = ubi
build-image-default: CONTAINER = $(APP_NAME)-$(BASE_IMAGE_TYPE)-build-1
build-image-default: BASE_IMAGE = $(UBI_BASE_IMAGE)
build-image-default: IMAGE_NAME = jostho/$(APP_NAME):v$(APP_VERSION)
build-image-default: LOCAL_BINARY_PATH = $(CURDIR)/target/release/$(APP_NAME)
build-image-default: LLVM_TARGET = $(shell $(RUSTC_PRINT_TARGET_CMD) | $(JQ_TARGET_CMD))
build-image-default: build-image

build-image-static: BASE_IMAGE_TYPE = scratch
build-image-static: CONTAINER = $(APP_NAME)-$(BASE_IMAGE_TYPE)-build-1
build-image-static: BASE_IMAGE = $(BASE_IMAGE_TYPE)
build-image-static: IMAGE_NAME = jostho/$(APP_NAME)-static:v$(APP_VERSION)
build-image-static: LOCAL_BINARY_PATH = $(CURDIR)/target/$(TARGET_MUSL)/release/$(APP_NAME)
build-image-static: LLVM_TARGET = $(shell $(RUSTC_PRINT_TARGET_CMD) --target $(TARGET_MUSL) | $(JQ_TARGET_CMD))
build-image-static: build-image

build-image:
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
		-l app-base-image=$(BASE_IMAGE_TYPE) \
		-l app-llvm-target=$(LLVM_TARGET) \
		$(CONTAINER)
	$(BUILDAH) commit --rm $(CONTAINER) $(IMAGE_NAME)
	$(BUILDAH) images

image: clean build prep-version-file get-flags build-image-default

image-static: clean build-static prep-version-file get-flags build-image-static

.PHONY: check check-required check-optional
.PHONY: clean prep-version-file get-flags
.PHONY: build build-static
.PHONY: build-image-default build-image-static build-image
.PHONY: image image-static
