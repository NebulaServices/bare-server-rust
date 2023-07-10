# bare-server-rust
bare-server-rust is a fully compliant Rust implementation of [TompHTTPs' Bare Server specifications](https://github.com/tomphttp/specifications/blob/master/BareServer.md).
This is a server that will receive requests from a service worker (or any client) and forward a request to a specified URL. 

## Using
TODO: Release builds to docker, create simple install script.

## Contributing 
All support and contributions to `bare-server-rust` are appreciated. Including, but not limited to: documentation changes, bugfixes, feature requests, and performance improvements.

### How do I get started?

### Before we start
A quick note before we start, we use unsatable features for rustfmt, requiring the +nightly flag. Make sure you install a nightly toolchain as well.

You can install the nightly rust toolchain with the following `rustup` command.
```
rustup toolchain install nightly
```

### Installing `rustup`
As with any rust project, you will need to install cargo, rust, and other dependencies. The easiest way to do this is with `rustup`.

If you are an a Unix based system, or intend to use Windows Subsystem for Linux to develop, you should run the `rustup` installer script below.
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Otherwise, please visit the [`Rustup Website`](https://rustup.rs/) to download it.

After you've finished with downloading rustup, be sure to install the nightly toolchain as shown [above](#before-we-start).
### Building from Sources
If you haven't done so already, you'll need to download the repository using git.
```
git clone git@github.com:NebulaServices/bare-server-rust.git
```
After you've download that, its time to go in and get to work. All you need to do is `cd` into the repository like so: 
```
cd bare-server-rust
```
Afterwords, you can either build or run using the respective cargo commands.
```
cargo run
cargo build
```
As an example, to build the `release` profile with the `v2` feature enabled, you can do it like so:
```
cargo run --features v2 --release 
```
## Authors
* [UndefinedBHVR](https://github.com/UndefinedBHVR) 
