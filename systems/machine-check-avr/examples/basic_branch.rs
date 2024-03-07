use machine_check::{Bitvector, BitvectorArray};
use machine_check_avr::ATmega328P;

fn main() {
    let hex = include_str!("basic_branch.hex");

    // fill with ones which is a reserved instruction
    let all_ones = Bitvector::new(0xFFFF);
    let mut progmem = BitvectorArray::new_filled(all_ones);

    machine_check_avr::read_hex_into_progmem(&mut progmem, hex);

    let system = ATmega328P { PROGMEM: progmem };
    machine_check::run(system);
}
