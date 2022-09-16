# Bracket QOS - Open Source Edition

> This project is based on the excellent [LibreQOS](https://github.com/rchac/LibreQoS) project. It is hoped that this can be useful for the upstream project.

**Please note that the Open Source port isn't quite ready for production yet. I'm still porting it over from our overly coupled to our network version. You should be able to get it to work, but there's quite a bit of work ahead for me --- documentation, cleaning up hard-coded paths and similar.**

## What is BracketQOS?

`BracketQOS` is a Rust implementation of [LibreQOS](https://github.com/rchac/LibreQoS), with a web front-end. It seeks to provide CAKE-based QoS/QoE for WISPs (Wireless Internet Service Providers), with an interface that helps your support team. We run it in production, and have been very happy with the overall results.

This is an Open Source port of our internal version (which is far too tied into how our network is setup). The port still needs quite a bit of work, but I wanted to make it available to the LibreQOS team as soon as possible. As time permits, we'll keep adding documentation, remove UISP as a requirement (and make it an option), improve the BPF side of things, etc.

## Components

* `qos_daemon` - runs on the shaping server, periodically updating queue trees from UISP and transmitting usage data to the QoS Manager.
* `qos_manager` - can (and probably should) be run on a different server, and provides a front-end to BracketQOS.

### Shared Library Modules

* `config` - stores the shaper configuration and handles serialization.
* `uisp_support` - limited UISP API implementation, enough to handle the queries required by BracketQOS. Currently, UISP is required. Removing UISP as a requirement --- and making it into an optional nicety --- is a priority.
* `shared_rest` - an API definition for the `qos_daemon` to talk to the `qos_manager`.

## Full Documentation

Please refer to the `docs` folder, which contains an `mdbook` book on how to setup `bracket-qos`.

## License

Because the original project is GPL, so is this one.
