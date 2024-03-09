#![doc = include_str!("../README.md")]

mod system;
mod util;

pub use system::machine_module::ATmega328P;
pub use system::machine_module::Input;
pub use system::machine_module::State;

pub use util::read_hex_into_progmem;
