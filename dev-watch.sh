#!/bin/bash

# Dependencies:
# cargo install systemfd cargo-watch

# from https://actix.rs/docs/autoreload/
touch '.trigger'
# kill both commands using trap https://unix.stackexchange.com/a/204619
trap 'kill %1' SIGINT
cargo watch -i .trigger -w src -w templates -x build -s 'touch .trigger' &
systemfd --no-pid -s http::8088 -- cargo watch -w .trigger -x "run start"
