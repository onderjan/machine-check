mod system;

/**
 * A system for [machine-check](https://crates.io/crates/machine-check) for checking machine-code programs for the AVR ATmega328P microcontroller.
 *
 * Known problems:
 * - Some lesser-used instructions are unimplemented.
 * - Only general-purpose I/O peripherals are supported.
 * - The program counter is not always checked for overflow.
 *
 * Inherently panics on:
 * - Jumps and calls outside program memory.
 * - Execution of reserved or illegal opcodes.
 * - Illegal or discouraged reads and writes.
 * - Push, pop, call, return with values read or written outside data memory.
 * - Unimplemented instructions, reads and writes.
 *
 * The system is written using the official instruction set reference
 * https://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf
 * and datasheet
 * https://ww1.microchip.com/downloads/aemDocuments/documents/MCU08/ProductDocuments/DataSheets/ATmega48A-PA-88A-PA-168A-PA-328-P-DS-DS40002061B.pdf
 */
pub use system::machine_module::ATmega328P;
pub use system::machine_module::Input;
pub use system::machine_module::State;
