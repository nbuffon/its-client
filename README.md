its-client
==========

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/Orange-OpenSource/its-client/workflows/Docker/badge.svg)](https://github.com/Orange-OpenSource/its-client/actions/workflows/docker.yml)

This Intelligent Transportation Systems (ITS) [MQTT][1] client based on
the [JSON][2] [ETSI][3] specification transcription provides a ready to connect project for the mobility
(connected and autonomous vehicles, road side units, vulnerable road users,...).

Let's connect your device or application to our Intelligent Transport Systems (ITS) platform!

Packages
--------

We provide many packages into the same project.

### JSon Schema

[ETSI.org][3] proposal of implementation using the [JSON][2] language (instead of ASN.1 by default).

### Rust

[![Build Status](https://github.com/Orange-OpenSource/its-client/workflows/Rust/badge.svg)][4]
[![crates.io](https://img.shields.io/crates/v/its-client)](https://crates.io/crates/its-client)

The Rust library to build a client.

### Python

#### iot3

Provides an abstraction of IoT3 for easy manipulation in Python.

#### its-quadkeys

[![Build status](https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-quadkeys.yml/badge.svg)][5]

Usefull abstractions around quadtrees, suitable for the ITS clients.

#### its-info

[![Build status](https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-info.yml/badge.svg)][6]

#### its-status

[![Build status](https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-status.yml/badge.svg)][7]

#### its-interqueuemanager

[![Build status](https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-iqm.yml/badge.svg)][8]

#### its-vehicle

[![Build status](https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-vehicle.yml/badge.svg)][9]


[1]: https://mqtt.org/
[2]: https://www.json.org
[3]: https://www.etsi.org/committee/its
[4]: https://github.com/Orange-OpenSource/its-client/actions/workflows/rust.yml
[5]: https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-quadkeys.yml
[6]: https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-info.yml
[7]: https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-status.yml
[8]: https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-iqm.yml
[9]: https://github.com/Orange-OpenSource/its-client/actions/workflows/python_its-vehicle.yml
