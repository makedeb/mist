#!/usr/bin/env -S just --justfile
set positional-arguments
export CARGO_RELEASE := ""

default:
    @just --list

build *ARGS:
    #!/usr/bin/env bash
    # Set Cargo args.
    if [[ "${CARGO_RELEASE:+x}" == 'x' ]]; then
        set -- "${@}" --release
        BIN_PATH='target/release/mist'
    else
        BIN_PATH='target/debug/mist'
    fi

    cargo build "${@}"

    if [[ "${NO_SUDO:+x}" == 'x' ]]; then
        sudo_cmd=''
    else
        sudo_cmd='sudo'
    fi

    ${sudo_cmd} chown 'root:root' "${BIN_PATH}"
    ${sudo_cmd} chmod a+s "${BIN_PATH}"

run *ARGS:
    #!/usr/bin/env bash
    just build
    target/debug/mist "${@}"