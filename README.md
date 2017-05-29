This is a template to make it easy to get started with live code reloading using BearLibTerminal.

Currently the live reloading is only tested/working on Linux, but the resulting program can be compiled in release mode for Linux and Windows. MacOS currently is untested.

## Installing required lib on Linux

This program relies on `libBearLibTerminal.so` so that should be copied into `usr/local/lib` or another folder indicated by this command: `ldconfig -v 2>/dev/null | grep -v ^$'\t'`

then you should run `sudo ldconfig` to complete the installation.

Then the executable should run correctly.

Alternately if your OS has a package for BearLibTerminal, that may work as well.

Once that's done compiling in debug mode with `cargo build` and release mode with `cargo build --release` should work.

## Compiling release mode for Windows

You will need a copy of the precompiled `BearLibTerminal.dll` and `BearLibTerminal.lib`.

Perform the folloing steps:

copy BearLibTerminal.lib to the project root

Comment out the line containing `crate-type = ["dylib"]` in the `Cargo.toml` in the `state_manipulation` folder. (this is more or less a workaround for [this issue](https://github.com/rust-lang/rust/issues/18807), hopefully we will eventually be able to make this switch using the `cfg` attribute, but currently using the attribute doesn't appear to work correctly.)

Run `cargo build --release` then copy the exe in `./target/release` to the desired location as well as `BearLibTerminal.dll` and any necessary assets (graphics, sound, etc.).
