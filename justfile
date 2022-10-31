#!/usr/bin/env just --justfile
# Some wrappers to aid in the build process during local development.
set positional-arguments

default:
    @just --list

build *args:
    cargo build "${@}"
    sudo chown 'root:root' target/*/mist
    sudo chmod a+s target/*/mist

run *args:
    just build
    target/debug/mist "${@}"