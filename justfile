default:
    just -l

test:
    cargo odra test -b casper

cli *ARGS:
    cargo run --bin styks-cli -- {{ARGS}}

build-guest-program:
    cd blocky-guest && make build

run-guest-program:
    cd blocky-guest && make run

build-website:
    cd styks-website && uv run convert.py

build-website-and-open: build-website
    open http://localhost:8000

run-website:
    cd styks-website/http-content && uv run -m http.server
