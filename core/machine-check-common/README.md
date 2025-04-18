
# Utility common-type crate for machine-check

This crate contains types used for communication between the formal
verification tool [machine-check](https://docs.rs/machine-check)
and its utility crates or programs that call its utility crates such as 
[machine-check-exec](https://docs.rs/machine-check-exec).
Note that directly using the utility crates should be avoided unless writing
an artefact that is concerned with the internal behaviour of a specific version
of [machine-check](https://docs.rs/machine-check).

## Usage and Compatibility

This crate is a utility crate for [machine-check](https://docs.rs/machine-check)
and should not be used on its own. No compatibility guarantees are made.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.
