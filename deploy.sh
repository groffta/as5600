#!/bin/bash
# This script expects an SSH entry 'bb' for deployment

CRATE_ROOT=$(dirname $0)

cargo build --target armv7-unknown-linux-gnueabihf
scp $CRATE_ROOT/target/armv7-unknown-linux-gnueabihf/debug/libas5600.so bb:~/.local/lib/libas5600.so
scp $CRATE_ROOT/as5600.py bb:~/as5600.py
ssh bb "python3 ~/as5600.py"