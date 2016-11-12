It's Terminal
=============
This is a terminal emulator. Why yet another? Because.

Installation
------------
To install it you will need a nightly Rust toolchain, then you can install it
with Cargo after cloning the repository.

```shell
cargo install --force
```

Terminfo
--------
You also have to run the following command to make sure the terminal
capabilities are installed.

```shell
cancer -T | tic -
```
