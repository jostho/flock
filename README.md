# Flock

![CI](https://github.com/jostho/flock/workflows/CI/badge.svg)

This is a flag quiz program written in rust using [rocket](https://github.com/SergioBenitez/Rocket).

## Environment

* fedora 32
* rustup 1.22
* rust 1.47.0-nightly
* make 4.2

## Build

To build or run, use cargo

    cargo build
    cargo run

## Run

Get `country-flags` sources from [here](https://github.com/hjnilsson/country-flags.git)

    git clone https://github.com/hjnilsson/country-flags.git

Start the binary - pass in the location of `country-flags` directory

    ./target/debug/flock -d ~/src/country-flags/

Open a browser - and access the application at `http://localhost:8000/`
