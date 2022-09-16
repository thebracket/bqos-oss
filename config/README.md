# BracketQOS Configuration

This crate handles loading/saving configurations. It assumes that configurations are stored in
`/usr/local/etc/bracket_qos.ron`.

An example configuration:

```ron
QosConfig(
        to_isp: "enp1s0f1",
        to_internet: "enp1s0f0",
        xdp_path: "/usr/local/xdp-cpumap-tc",
        internet_upload_mbps: 2000,
        internet_download_mbps: 2000,
        default_upload_mbps: 5,
        default_download_mbps: 10,
        nms_key: "<put your NMS key in here>",
        nms_url: "https://billing.myisp.com/",
        root_site_name: "ROOT_SITE_NAME",
        strategy: Full,
        include_ip_ranges: [ "172.16.0.0/12", "10.0.0.0/8", "100.64.0.0/10", "192.168.0.0/16", ],
        ignore_ip_ranges: [ "192.168.15.0/24", ],
        controller_url: "http://<ip address>:9123",
)
```
