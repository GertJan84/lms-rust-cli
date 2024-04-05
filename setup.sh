#!/bin/bash

cargo build --release
mv target/release/lms ~/.local/bin

echo "Update local production envirement"