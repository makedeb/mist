build:
    cargo build

test-root:
    just build

    sudo chown root target/debug/mist
    sudo chmod a+s target/debug/mist

    if ./target/debug/mist search nonexistent; then echo "Missing 'search' results should not exit with status code 0" && exit 1; fi

    if ./target/debug/mist list nonexistent; then echo "Missing 'list' results should not exit with status code 0" && exit 1; fi