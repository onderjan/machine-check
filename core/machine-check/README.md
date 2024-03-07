# `Machine-check`: a formal verification tool for digital systems

`Machine-check` is a tool for formal verification of digital system properties, currently 
in experimental stage.

Unlike classic software testing, which can find bugs but not prove their absence,
formal verification can prove that specified undesirable behaviours can never occur
in the system. Unfortunately, formal verification requires complicated reasoning 
(e.g. with intervals instead of just numbers) and advanced techniques are necessary 
for its automation. `Machine-check` aims to provide these techniques, but shield 
the user from them as much as possible.

The main intended use-case of `machine-check` is formal verification of machine-code programs 
on simple microcontrollers, but the approach is highly general, allowing verification 
of arbitrary digital systems as long as they are described in a subset of valid Rust code 
that `machine-check` understands.

## Examples
A very simple example of a system verifiable by `machine-check` is [counter](
    https://docs.rs/crate/machine-check/0.2.0-alpha.1/source/examples/counter.rs), 
a simple [finite-state machine](https://en.wikipedia.org/wiki/Finite-state_machine) which contains 
an eight-bit `value`, which is initialized to zero on initialization and then is incremented 
if the `increment` single-bit input is 1. If the value reaches 157, it is immediately zeroed again. 
The system is very simple, so it is complicated a little by a large unused bitvector array,
which would make simple kinds of automated verification impossible.

The magic of `machine-check` is unlocked by the [`machine_description`] macro, which adds verification 
analogues to the code it is applied to, allowing simply running `machine-check` by calling [`run`] in the main 
function after constructing the system.

If you put the [counter](examples/counter.rs) inside your own Rust crate (with `machine-check` as a dependency)
, you can  verify that the counter is always lesser than 157 in each system state, using a specification property 
based on [Computation Tree Logic](https://en.wikipedia.org/wiki/Computation_tree_logic). Let's use 
`machine-check` to prove the property: 
```
$ cargo run -- --property "AG[unsigned_lt(value,157)]"

[2024-03-07T22:44:44Z INFO  machine_check_exec] Starting verification.
[2024-03-07T22:44:44Z INFO  machine_check_exec] Verification ended.
{"result":{"Ok":true},"stats":{"num_states":178,"num_refinements":309}}
```
(Note that the final specification and output formats are still under construction 
and will be nicer than this.)

On the other hand, `machine-check` tells us that the counter value is NOT always lesser than 156:
```
$ cargo run -- --property "AG[unsigned_lt(value,156)]"

$ cargo run --example counter -- --property "AG[unsigned_lt(value,156)]"
[2024-03-07T22:45:47Z INFO  machine_check_exec] Starting verification.
[2024-03-07T22:45:47Z INFO  machine_check_exec] Verification ended.
{"result":{"Ok":false},"stats":{"num_states":178,"num_refinements":308}}
```

You can use the "-v" command-line parameter to show the final abstract state space, although interpreting it
requires some further knowledge.

### Inherent panics
It is also possible to detect system panics, which is useful e.g. for machine-code systems, which
take the machine-code file from command line and should detect that it a reserved instruction
can be executed during the course of the program. A simple example is in the 
[conditional_panic](
    https://docs.rs/crate/machine-check/0.2.0-alpha.1/source/examples/conditional_panic.rs) 
example, which should panic with message 
"Test panic 2" if the input is equal to 1. You can copy it into your crate, then run it with 
parameter "--inherent", which signifies that you are only interested about the inherent 
panics of the system:
```
$ cargo run --example conditional_panic -- --inherent
[2024-03-07T22:59:26Z INFO  machine_check_exec] Starting verification.
[2024-03-07T22:59:26Z ERROR machine_check_exec] Verification failed.
{"result":{"Err":{"InherentPanic":"Test panic 2"}},"stats":{"num_states":3,"num_refinements":8}}
```
Currently, presence of inherent panic precludes verification of a property, as it is 
a more pressing issue to fix.

### Machine-code verification
There is also an example of an extremely simplified RISC microcontroller 
in [simple_risc](https://docs.rs/crate/machine-check/0.2.0-alpha.1/source/examples/simple_risc.rs), 
showcasing the [`bitmask_switch`] macro that 
can be used for elegant transcription of microcontroller behaviour depending on instruction 
opcodes.

A more proper implementation of an actual microcontroller is present in the crate 
[machine-check-avr](https://crates.io/crates/machine-check-avr), allowing verification 
of some simple machine-code programs for the AVR ATmega328P microcontroller 
(notably used in Arduino Uno R3).


## Current status
`Machine-check` is still in experimental phase, with limitations in user experience 
and verification power.

The [`machine_description`] macro, in particular, is currently very finicky and errors
produced may or may not be useful. If you want to try writing your own system, proceed 
step by step, slowly adding and modifying pieces of example code. Temporarily commenting out 
the macro may also reveal an underlying Rust error with a more sensible error message.

## Further notes
Unlike some other formal verification tools, `machine-check` is designed be sound 
and complete, i.e. you should either get an error or a correct true/false result in finite 
(but practically unbounded) time. Of course, there may be bugs or design oversights.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.
