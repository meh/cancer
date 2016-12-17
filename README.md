It's Terminal
=============
This is a terminal emulator. Why yet another? Because.

Installation
------------
To install it you will need [rustup.rs](https://rustup.rs).

```shell
curl https://sh.rustup.rs -sSf | sh
rustup install nightly
```

Then you can install it with Cargo.

```shell
cargo +nightly install --force --git https://github.com/meh/cancer
```

You also have to run the following command to make sure the terminal
capabilities are installed.

```shell
cancer -T | tic -
```
