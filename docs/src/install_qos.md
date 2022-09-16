# Installing the QOS Daemon

The QOS Daemon (`qos_daemon`) runs continually on your QoS server. The server must have a network interface facing the Internet, and a second NIC facing your local network. A third NIC for management is a good idea. It can be a physical or virtual server.

## OS Setup

1. Install Ubuntu Server. We currently use version 21.10.
2. Ensure that you have kernel version 5.13 or later (is you are in a VM), or 5.11 or later on a physical host.

### Setup a Bridge

Find out which interfaces you will be using for Internet and Internal traffic. Note these down.

Then open `/etc/netplan/00-installer-config.yaml`. At the bottom of the file, add a bridge containing these two interfaces:

```yaml
# This is the network config written by 'subiquity'
network:
  ethernets:
    ens18:
      addresses:
      - 172.16.172.49/24
      gateway4: 172.16.172.1
      nameservers:
        addresses:
        - 172.16.172.16
        - 1.1.1.1
        search: []
    ens19:
      dhcp4: no
    ens20:
      dhcp4: no
    enp1s0f0:
      dhcp4: no
    enp1s0f1:
      dhcp4: no
  version: 2
  bridges:
    br0:
      interfaces:
        - enp1s0f0
        - enp1s0f1
```

You shouldn't need to change anything except the `bridges` section.

Then run `sudo netplan apply` to enable the bridge.

## Setup Required Components

### xdc-cpumap-tc

`xdc-cpumap-tc` is a project that improves overall throughput of Linux traffic shaping by mapping queues to CPUs --- freeing you from Linux's "one CPU does all the queueing" issue.

> It's on the roadmap to replace this with a more flexible version.

Clone the `xdc-cpumap-tc` project to your server. I recommend putting it in `/usr/local/xdp-cpumap-tc`:

```
git clone https://github.com/xdp-project/xdp-cpumap-tc.git /usr/local/xdp-cpumap-tc
```

Once it is downloaded, you have to compile it.

> Note that you need to have a development environment working on your server.

```
cd /usr/local/xdp-cpumap-tc/src
make
```

### pping

BracketQOS uses `pping` to continually monitor TCP latency on your network. It doesn't work like `ping` - it monitors actual customer TCP traffic, timing the latency of each individual connection.

```
cd ~
git clone https://github.com/pollere/pping.git
cd pping
git clone https://github.com/mfontanini/libtins.git
cd libtins
sudo apt-get install libpcap-dev libssl-dev cmake
mkdir build
cd build
cmake ../ -DLIBTINS_BUILD_SHARED=0 -DLIBTINS_ENABLE_CXX11=1 \
 -DLIBTINS_ENABLE_ACK_TRACKER=0 -DLIBTINS_ENABLE_WPA2=0 \
 -DCMAKE_INSTALL_PREFIX=`dirname $PWD`
make
make install
cd ..
cd ..
```

Once it has compiled, copy it to `/usr/local/bin` with `cp pping /usr/local/bin`.

### Install a Rust Toolchain

Visit [https://rustup.rs/](https://rustup.rs/) and follow the instructions to install the Rust toolchain.

## Install the QoS Daemon

Go to your home directory, and clone the `bracket-qos` repository:

```
cd ~
git clone https://github.com/thebracket/bqos-oss.git
```

Now enter the `qos_daemon` directory and build the project:

```
cd bqos-oss/qos_daemon
cargo build --release
```

Copy the resulting binary to your `/usr/local/bin` folder:

```
cp ../target/release/qos_daemon /usr/local/bin
```

## Configure BracketQOS

Create a new file, `/usr/local/etc/bracket_qos.ron`:

```
nano /usr/local/etc/bracket_qos.ron
```

The configuration file needs the following information added to it:

```ron
QosConfig(
        to_isp: "enp1s0f1",
        to_internet: "enp1s0f0",
        xdp_path: "/usr/local/xdp-cpumap-tc",
        internet_upload_mbps: 2000,
        internet_download_mbps: 2000,
        default_upload_mbps: 5,
        default_download_mbps: 10,
        nms_key: "<your UISP key>",
        nms_url: "<your UISP URL>",
        root_site_name: "<the UISP name of your site connected to the Internet. Case sensitive.>",
        strategy: Full,
        include_ip_ranges: [ "172.16.0.0/12", "10.0.0.0/8", "100.64.0.0/10", "192.168.0.0/16", "64.195.0.0/19", "216.106.34.0/24" ],
        ignore_ip_ranges: [ "192.168.15.0/24", "192.168.2.0/24", "192.168.1.0/24", "192.168.0.0/24", "192.168.16.0/24", "192.168.10.0/24" ],
        controller_url: "http://172.16.10.212:9123",
)
```

The following items are required:

* `to_isp`: *must* contain the name of the interface facing the Internet (one of the two you added to your bridge).
* `to_internet`: *must* contain the name of the interface facing your network (the other bridge entry).
* `xdp_path`: where you installed `xdp-cpumap-tc`. If you followed these instructions, `/usr/local/xdp-cpumap-tc`
* `internet_upload_mbps`: the total amount of Internet bandwidth you have available (upload) - in Megabits per Second.
* `internet_download_mbps`: the total amount of Internet bandwidth you have available (download) - in Megabits per Second.
* `default_upload_mbps`: the upload speed (in Megabits per Second) to apply to unmapped customers.
* `default_upload_mbps`: the download speed (in Megabits per Second) to apply to unmapped customers.
* `nms_key`: visit UISP, go to "settings" and "users". Create a new API token, and paste it in here.
* `nms_url`: the URL of your UISP installation. For example: `https://billing.myisp.com/`. Include the last slash, don't try and include API locations --- it will find them for you.
* `root_site_name`. In UISP, one site is connected to the Internet. This is your "root" site. Copy its name (case sensitive) into this entry.
* `strategy`. This can be `Full` (offering site, AP and client level shaping), `SiteOnly` (offering a queue per site and then a queue per customer underneath it), or `JustClients` --- which just generates a queue per customer.
* `include_ip_ranges`: list all of the subnets in which your clients reside.
* `ignore_ip_ranges`: this is applied *after* included ranges, so you can carve out chunks of included ranges to ignore. Any IPs in these ranges will be ignored for queue creation and reports. They will still be placed in the default queues if they communicate with the Internet.

Once that's complete, you are ready to try the shaper.

## Run the Shaper Daemon

Execute:

```
/usr/local/bin/qos_daemon
```

You should see the daemon connect to UISP, download your network topology and create queues. You won't have any per-site or per-AP shaping yet (it's there, but without speed limits).

Take a look at `/usr/local/etc/last_known_good_tree.ron` to see what's going on. Here's part of an example (with customer names removed):

```ron
(
    queue_count: (
        to_isp: 9,
        to_internet: 9,
    ),
    ip_to_site_map: {},
    queues: [
        (
            name: "CPU 1 Queue",
            queue_type: CpuQueue(
                cpu_id: 1,
            ),
            children: [
                (
                    name: "Medusa East",
                    queue_type: AccessPointSite(
                        site_id: "062a7c9c-2ea0-4899-8c95-bc31235b4baf",
                        down_mbps: 2000,
                        up_mbps: 2000,
                    ),
                    children: [
                        (
                            name: "A client",
                            queue_type: ClientSite(
                                site_id: "f7dc8d53-af89-4059-9c71-9fa6e07cf303",
                                down_mbps: 10,
                                up_mbps: 5,
                                ip_addresses: [
                                    "172.29.201.14",
                                ],
                            ),
                            children: [],
                        ),
                        .. (etc)
```
