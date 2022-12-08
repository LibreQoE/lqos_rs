# LQOSD

**The LibreQoS Daemon** is designed to run as a `systemd` service at all times. It provides:

* Load/Unload the XDP/TC programs (they unload when the program exits)
* Configure XDP/TC, based on the content of `ispConfig.py`.
   * Includes support for "on a stick" mode, using `OnAStick = True, StickVlanA = 1, StickVlanB = 2`.
* Hosts a lightweight server offering "bus" queries for clients (such as `lqtop` and `xdp_iphash_to_cpu_cmdline`).
   * See the `lqos_bus` sub-project for bus details.
* Periodically gathers statistics for distribution to other systems via the bus.
