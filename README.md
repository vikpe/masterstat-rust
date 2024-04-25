# masterstat

> Get server addresses from QuakeWorld master servers.

## Installation

```shell
cargo add masterstat
```

## Usage

**Query a single master server**

```rust
use std::time::Duration;

let master = "master.quakeworld.nu:27000";
let timeout = Some(Duration::from_secs(2));
let server_addresses = masterstat::server_addresses(&master, timeout)?;
```

**Query multiple master servers (async, in parallel)**

```rust
use std::time::Duration;

let masters = ["master.quakeworld.nu:27000", "master.quakeservers.net:27000"];
let timeout = Some(Duration::from_secs(2));
let server_addresses = masterstat::server_addresses_from_many(&masters, timeout).await?;
```

## See also

* [masterstat](https://github.com/vikpe/masterstat) (golang version)
* [masterstat-cli](https://github.com/vikpe/masterstat-cli) - CLI for fetching server addresses from QuakeWorld master
  servers.
