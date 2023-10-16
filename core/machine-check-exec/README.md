# Utility executable-logic crate for machine-check

This crate contains machine verification logic for the formal verification tool
[machine-check](https://crates.io/crates/machine-check). In essence, [machine-check](
https://crates.io/crates/machine-check) generates a Rust crate with machine behaviour
translated to Rust with use of types and operations in another utility crate
[mck](https://docs.rs/mck). The main entry point of the generated binary
just calls the [run](run) function in this crate, which contains the actual
verification logic.

# Usage and Compatibility

This crate is a utility crate for [machine-check](https://crates.io/crates/machine-check)
and should not be used on its own. No compatibility guarantees are made.

# License

This crate is licensed under Apache 2.0 License or MIT License at your discretion.
