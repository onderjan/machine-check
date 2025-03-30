# **Machine-check**: a formal verification tool for digital systems

**Machine-check** is a formal verification tool usable for e.g. machine-code 
or hardware verification.
- [Website](https://machine-check.org)
- [User guide](https://book.machine-check.org)
- [GitHub Repository](https://github.com/onderjan/machine-check)

Unlike classic software testing, which can find bugs but not prove their absence,
formal verification can prove that specified undesirable behaviours can never occur
in the system. While formal verification requires complicated reasoning, **machine-check**
can verify arbitrary systems described as finite-state machines automatically.

The main intended use-case of **machine-check** is formal verification of machine-code programs 
on microcontrollers. However, the approach is general and allows verification 
of arbitrary digital systems as long as they are described in a subset of valid Rust code 
that **machine-check** understands.

## A Quick Example

The magic of **machine-check** is unlocked by the [`machine_description`] macro, which adds verification 
analogues to the code it is applied to. You can then run **machine-check** from your own crate by 
constructing the system and providing it to the function [`run`].

A very simple example of a system verifiable by **machine-check** is 
[counter](https://docs.rs/crate/machine-check/0.4.0/source/examples/counter.rs), 
a simple [finite-state machine](https://en.wikipedia.org/wiki/Finite-state_machine) which contains 
an eight-bit state field `value`, which is initialized to zero and then is incremented in each step exactly
if the single-bit input `increment` is set (1). If the value reaches 157, it is immediately zeroed. 
The system is very simple, so it is complicated a little by a large unused bitvector array,
which would make simple kinds of automated formal verification impossible.

To try things out, create a new package:
```console
$ cargo new my-example --bin
    Creating binary (application) `my-example` package
    (...)
$ cd my-example
```
Copy the source code of [counter](https://docs.rs/crate/machine-check/0.4.0/source/examples/counter.rs) 
to `src/main.rs` and add **machine-check** as a dependency to `Cargo.toml`:
```toml
[dependencies]
machine-check = "0.4.0"
```
We can then verify that the counter is lesser than 157 in every reachable system state, 
using a specification property based on 
[Computation Tree Logic](https://en.wikipedia.org/wiki/Computation_tree_logic):
```console
$ cargo run --release -- --property 'AG![as_unsigned(value) < 157]'
   Compiling autocfg v1.4.0
   (...)
   Compiling machine-check v0.4.0
   Compiling my-example v0.1.0 ({your_path}\my-example)
    Finished `release` profile [optimized] target(s) in 21.49s
     Running `target\release\my-example.exe --property "AG![as_unsigned(value) < 157]"`
[2025-03-29T23:51:38Z INFO  machine_check] Starting verification.
[2025-03-29T23:51:38Z INFO  machine_check::verify] Verifying the inherent property first.
[2025-03-29T23:51:38Z INFO  machine_check::verify] The inherent property holds, proceeding to the given property.
[2025-03-29T23:51:38Z INFO  machine_check::verify] Verifying the given property.
[2025-03-29T23:51:39Z INFO  machine_check] Verification ended.
+--------------------------------+
|         Result: HOLDS          |
+--------------------------------+
|  Refinements:             157  |
|  Generated states:        471  |
|  Final states:            157  |
|  Generated transitions:   628  |
|  Final transitions:       315  |
+--------------------------------+
```
We were able to determine that the counter is lesser than 157 in every reachable state using **machine-check**.
We can also be informed that the value is **NOT** lesser than 156 in every reachable state:
```console
$ cargo run --release -- --property 'AG![as_unsigned(value) < 156]'
    Finished `release` profile [optimized] target(s) in 0.15s
     Running `target\release\my-example.exe --property "AG![as_unsigned(value) < 156]"`
[2025-03-29T23:54:17Z INFO  machine_check] Starting verification.
[2025-03-29T23:54:17Z INFO  machine_check::verify] Verifying the inherent property first.
[2025-03-29T23:54:17Z INFO  machine_check::verify] The inherent property holds, proceeding to the given property.
[2025-03-29T23:54:17Z INFO  machine_check::verify] Verifying the given property.
[2025-03-29T23:54:18Z INFO  machine_check] Verification ended.
+--------------------------------+
|     Result: DOES NOT HOLD      |
+--------------------------------+
|  Refinements:             156  |
|  Generated states:        470  |
|  Final states:            161  |
|  Generated transitions:   626  |
|  Final transitions:       318  |
+--------------------------------+
```

See the [website](https://machine-check.org) and [user guide](https://book.machine-check.org)
for more information.

## Machine-code Verification

The crate [machine-check-avr](https://docs.rs/machine-check-avr) includes a system description
of the AVR ATmega328P microcontroller (notably used in Arduino Uno R3), allowing verification
of simple machine-code programs. More systems may come later.

## Current Status

**Machine-check** is still in developmental phase, with limitations in user experience 
and verification power. There may (and probably will be) some bugs or design oversights.
Bug reports to the [repository](https://github.com/onderjan/machine-check) are very welcome.

## Minimum Supported Rust Version
The Minimum Supported Rust Version for **machine-check** `0.4.0` is `1.83`.
This can be raised to any stable Rust version in a minor release.
The MSRV will not be raised in a patch release.

## Changelog
 - `0.4.0`: An initial version of a Graphical User Interface, a monotonicity fix,
   tweaks to the verification core including no longer short-circuiting
   state generation on panic when verifying the inherent property.
 - `0.3.1`: Each refinement continues until the state space changes. This improves 
   performance a bit in some scenarios.
 - `0.3.0`: Soundness fixes, optimisation, refinement choice tweaks for reasonable
   verification of machine-code systems.
 - `0.2.0`: A significant rewrite, arbitrary finite-state systems now can be described 
   as finite-state machines in Rust code. Conditional branches are supported.
 - `0.1.0`: Initial version, only verification of 
   [Btor2](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_32) hardware 
   systems supported through translation to Rust code.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.
