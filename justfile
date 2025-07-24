default:
    just -l

test:
    cargo odra test

cli *ARGS:
    cargo run --bin styks-cli -- {{ARGS}}

build-guest-program:
    cd blocky-guest && make build

run-guest-program:
    cd blocky-guest && make run