It's Terminal
=============
This is a terminal emulator. Why yet another? Because.

Installation
------------
To install it you will need a nightly Rust toolchain, you should probably use
[rustup](http://rustup.rs) to get it.

```shell
cargo install --force --git https://github.com/meh/cancer
```

You also have to run the following command to make sure the terminal
capabilities are installed.

```shell
cancer -T | tic -
```

Philosophy
==========

Architecture
============

Configuration
=============
The default path for the configuration file is platform dependant, on Linux it
will use `$XDG_CONFIG_PATH/cancer/config.toml`.


`[environment]`
---------------
The `environment` block is used for system specific configurations.

+-----------+---------------------------------------------------+
| `display` | On X11 systems it specifies the `DISPLAY` to use. |
+-----------+---------------------------------------------------+

Key bindings
============
