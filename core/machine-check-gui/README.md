# Utility machine manipulation crate for machine-check

This crate implements the graphical user interface (GUI) for use in the formal verification tool 
[machine-check](https://docs.rs/machine-check). The GUI is an optional feature intended for 
deepening the understanding of the systems, specifications, and the verification process.
Machine-check can also be used headless using the command-line interface only.

## Implementation Details
The GUI is based on a WebView, i.e. uses the browser installed on the machine to show the HTML/CSS 
GUI in a native-style application. This has the advantage of rapid development compared to 
a non-standard UI library and of small executable size compared to the Electron-style approach
where the browser is bundled in the executable. However, it may suffer from compatibility quirks 
and issues.

The frontend is not implemented in Javascript, but in Rust compiled to WebAssembly, interacting with 
the Rust-based backend running natively. This speeds up development and prevents language-interaction
bugs, but makes building the GUI more tricky: a `wasm32-unknown-unknown` Rust target is needed to build 
the frontend, which is usually done in a temporary directory. A compatible `wasm-bindgen` is also needed.

For these reasons, in the officially released versions, the compiled WebAssembly is already provided,
which makes it possible to forgo installing the additional target and `wasm-bindgen` when only using
[machine-check](https://docs.rs/machine-check) as a library.

## Usage and Compatibility

This crate is a utility crate for [machine-check](https://docs.rs/machine-check)
and should not be used on its own. No compatibility guarantees are made.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.
