# Utility type/operation crate for machine-check

This crate is used to define concrete, abstract, etc. types and operations
for the formal verification tool [machine-check](
https://crates.io/crates/machine-check). As [machine-check](
https://crates.io/crates/machine-check) generates Rust code using these types
and operations and then builds and executes the resulting files, the name of
this crate was chosen so the generated absolute paths are as concise as possible.

# Usage and Compatibility

This crate is a utility crate for [machine-check](https://crates.io/crates/machine-check)
and should not be used on its own. No compatibility guarantees are made.

# License

This crate is licensed under Apache 2.0 License or MIT License at your discretion.
