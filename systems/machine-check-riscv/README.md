# RISC-V microcontroller machine-code verification using machine-check

The executable in this crate allows formal verification of machine-code 
programs for the [Renesas R9A02G021CNE](https://www.renesas.com/en/products/r9a02g021)
microcontroller (as used in the 
[Renesas FPB-R9A02G021](https://www.renesas.com/en/design-resources/boards-kits/fpb-r9a02g021)
prototyping board) with a 32-bit RISC-V core.

See the **machine-check** [book](https://book.machine-check.org) for details. 
Note that both **machine-check** and this crate are currently in developmental phase.

## Usage

In addition to common **machine-check** executable arguments,
the executable takes a flag specifying path to an ELF format file 
containing the program code: `--system-elf-file abc.elf` (or just `-E abc.hex`).

## Custom macros

There are three custom macros that can be used in properties, all of which
take one argument with a symbol name (where the symbol with the name is
present in the ELF file exactly once):

- `pc_at_symbol`: produces a Boolean expression that determines whether the
Program Counter is at the address of a symbol (for symbols with an address,
typically e.g. labels) or at the start of the address range (for symbols 
with an address range, typically e.g. functions).
- `pc_within_symbol`: produces a Boolean expression that determines whether the
Program Counter is within the symbol address range (for symbols with an address range,
typically e.g. functions).
- `typed_symbol`: produces a [`machine_check::Bitvector`] containing the value
of a symbol which has a type sized in bytes. For example, if there is a global 
variable `example_variable` of `uint32_t` type, the macro 
`typed_symbol("example_variable")` will produce a 32-bit 
[`machine_check::Bitvector`] with the value of the variable.

A verification error will be produced if the symbol was not parsed as the correct kind.

Usable symbols will be printed when debug prints are enabled (`-v` flag).

## Known system inaccuracies

- Currently, the system is proof-of-concept, with many possibilities for bugs.
- RV32IC is implemented but other extensions supported by R9A02G021 are not.
- Only a very small subset of general-purpose I/O, Control Status Registers
    and other registers is supported.

## Inherent panics

- Non-aligned loads and stores (not supported by R9A02G021).
- Execution of reserved or illegal opcodes (either as `unimplemented` or `panic`).

## Used resources

The system is written using the official RISC-V Instruction Set Manual Volume I 
(Unprivileged Architecture) and 
[R9A02G021 User Manual](https://www.renesas.com/en/document/mah/r9a02g021-users-manual-hardware).

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.
