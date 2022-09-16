# Introduction

BracketQOS is a Quality of Experience system designed for Wireless Internet Service Providers. It is similar to Preseem: it provides QoS/QoE at the edge of your network, loading your network topology from your network management software (currently UISP). Each customer has a CAKE queue applied to them, which both limits their available bandwidth to match their billing plan and actively monitors current QoE (especially latency) and shapes individual streams within the client's queue to optimize their perceived performance.

> BracketQOS is based on the excellent [LibreQOS](https://github.com/rchac/LibreQoS) project.

## What do I need?

At a minimum, you need:

* One server with a NIC facing the Internet, and another NIC facing your internal network. This can be a virtual server (we use ProxMox for this) or a physical server. Physical servers have significantly better overall performance - but you can easily shape 2 Gbps on a decent-sized virtual server.

We recommend splitting the software between two servers:

* A QoS server with a NIC facing the Internet, and another NIC facing your internal network.
* A manager server running the `qos_manager` monitoring software. You also need `influxdb2` installed, either on this server or a third server. InfluxDB can be very write-heavy, so putting it on its own server with fast storage can greatly improve your overall experience.

On the QoS server, you need:

* A Rust toolchain to build/install the setup.
* [xdp-cpumap](https://github.com/xdp-project/xdp-cpumap-tc/tree/888cc7712f2516d386a837aee67c5b05bd04edfa) compiled and ready to use.
* [pping](https://github.com/pollere/pping) installed in `/usr/local/bin/pping`.

* On the management server, you need:

* A Rust toolchain to build/install the setup.
* InfluxDB 2, either locally installed or on another server.

##