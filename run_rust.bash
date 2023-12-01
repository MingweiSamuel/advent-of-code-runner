#!/bin/bash
# Usage: `$0 <USER> <DAY> [INPUT]`
# Example: `$0 mingweisamuel 5 example`
set -eux

INPUT=${3:-$1}
cat "2022/$2/io/$INPUT.input" | RUST_BACKTRACE=1 cargo run --manifest-path "2022/$2/solutions/rust/$1/Cargo.toml"
