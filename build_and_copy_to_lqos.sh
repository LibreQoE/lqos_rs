#!/bin/bash
for prog in lqosd lqtop xdp_iphash_to_cpu_cmdline xdp_pping
do
    pushd $prog
    cargo build --release
    popd
done
sudo cp target/release/xdp_iphash_to_cpu_cmdline /opt/libreqos/v1.3/
