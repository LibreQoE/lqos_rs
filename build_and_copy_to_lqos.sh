#!/bin/bash
cargo build --all --release
sudo cp target/release/xdp_iphash_to_cpu_cmdline /opt/libreqos/v1.3/
