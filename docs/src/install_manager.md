# Install the Manager

If you chose to use a second server, you'll need to install the Rust toolchain on that server also.

## Install Influx DB 2

I used the Ubuntu crate to install `influxdb2`: `apt install influxdb2`. Once its installed, go to its configuration system in a web browser (e.g. `http://<ip address>:8086/signin`). Log in. In the `data` section, create a bucket for your data. I named mine `bracketqos`. You can use whatever name you like. It's a good idea to set a retention policy---it can use a lot of space.

## Install the Manager

Clone the `bracket-qos` repo (if you haven't already, because you are using a second server):

```
cd ~
git clone https://github.com/thebracket/bqos-oss.git
```

Now build the manager:

```
cd bqos-oss/qos_manager
cargo build --release
```

### Create a Configuration File

In `~/bqos-oss/qos_manager` create a new file named `qos_manager.ron`. It needs to know how where everything is:

```ron
QosManagerConfig(
    influx_url : "http://<address of Influx database>:<port number of Influx, by default 8086>",
    influx_org : "<Your Influx organization name>",
    influx_token : "<your Influx token>",
    influx_bucket : "<your Influx bucket>",
    nms_key: "<your UISP key, same as in the qos daemon>",
    nms_url: "https://<your UISP URL>/nms/api/v2.1",
    crm_key: "<Another UISP key, generated in the CRM side>",
    crm_url: "https://<your UISP server>/api/v1.0",
)
```

## Run the manager

Execute `cargo run --release` and login to `http://<ip>:9123/`. Make sure that your QOS Daemon config knows where this server is (in its configuration file), and restart it.

After a minute or two, the `qos_manager` will show you your network and begin collecting data.
