# Flock

![CI](https://github.com/jostho/flock/workflows/CI/badge.svg)
![Image](https://github.com/jostho/flock/workflows/Image/badge.svg)

This is a flag quiz program written in rust using [rocket](https://github.com/SergioBenitez/Rocket).

## Environment

* fedora 36
* rustup 1.24
* rust 1.61
* make 4.3

## Build

To build, use `cargo`

    cargo build

## Run

Get `country-flags` sources from [here](https://github.com/hampusborgos/country-flags)

    make get-flags

Start the binary - pass in the location of `country-flags` directory

    ./target/debug/flock -d ./target/country-flags-main/

Open a browser - and access the application at `http://localhost:8000/`

## Image

A `Makefile` is provided to build a container image

Check prerequisites to build the image

    make check

To build the default container image

    make image

To run the container image - use `podman`

    podman run -d -p 8000:8000 <imageid>
