# Qos Manager

This is the web interface that a) receives data from the `qos_daemon` and stores it for monitoring, and b) provides an interactive view of your network's QoS/QoE in near real-time.

This is a stripped down version of the system we use in production; our version is a little too tied into our setup to share verbatim. (Our version includes integration with how we handle cnMaestro, and DHCP-based static address assignment)

## Setup

The `qos_manager` expects to find a configuration file in its working directory. The file must be named `qos_manager.ron`. An example configuration file looks like this:

```ron
QosManagerConfig(
    influx_url : "http://<address of your Influx DB 2 server>:<port number, usually 8086>",
    influx_org : "<name of your Influx DB 2 data store>",
    influx_token : "<token created with Influx DB 2 for remote access>",
    influx_bucket : "<bucket name>",
    nms_key: "<key of your UISP NMS access token>",
    nms_url: "https://<address of UISP>/nms/api/v2.1",
    crm_key: "<API key for the CRM side of your UISP setup>",
    crm_url: "https://<address of your UISP>/api/v1.0",
)
```

You also need to have `Rocket.toml` in the daemon's working directory. This sets up the IP address and port number on which the service should listen. There's a default in the repository. It looks like this:

```toml
[default]
port = 9123
address = "0.0.0.0"
```

Once all of that is in place, you can run the `qos_manager`. Either:

* Run `cargo build --release` and copy `target/release/qos_manager` to a folder containing these files. This folder must also contain a copy of the `static` folder. **OR**
* Execute `cargo run --release` with the files in the `qos_manager` directory of the checked-out project.

You can then navigate to `https://<ip>:<port>/`.