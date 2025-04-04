# AVR microcontroller machine-code verification using machine-check

The executable in this crate allows formal verification of machine-code 
programs for the AVR ATmega328P microcontroller via [machine-check](https://docs.rs/machine-check).

In addition to common [machine-check](https://docs.rs/machine-check) executable arguments,
the executable takes a flag specifying path to Intel HEX file 
containing the ATmega328P program code: `--system-hex-file abc.hex` (or just `-H abc.hex`).

See [machine-check](https://docs.rs/machine-check) for details on verifying specifications.

Note that both [machine-check](https://docs.rs/machine-check) and this crate are currently 
in developmental phase and awaiting further improvement.

## Known system problems

- Some lesser-used instructions are unimplemented.
- Only general-purpose I/O peripherals are supported.
- The program counter may not always be checked for overflow.

## Inherent panics

- Jumps and calls outside program memory.
- Execution of reserved or illegal opcodes.
- Illegal or discouraged reads and writes.
- Push, pop, call, return with values read or written outside data memory.
- Unimplemented instructions, reads and writes.

## Used resources

The system is written using the official [AVR instruction set manual](
https://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf)
and [non-automotive ATmega328P datasheet](
https://ww1.microchip.com/downloads/aemDocuments/documents/MCU08/ProductDocuments/DataSheets/ATmega48A-PA-88A-PA-168A-PA-328-P-DS-DS40002061B.pdf).

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be 
dual licensed as above, without any additional terms or conditions.
