use clap::Args;
use machine_check::{Bitvector, BitvectorArray};

#[derive(Args)]
pub struct SystemArgs {
    #[arg(long)]
    hex_file: String,
}

fn main() {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());

    let hex = match std::fs::read_to_string(system_args.hex_file) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("Could not read hex file: {}", err);
            return;
        }
    };

    // fill with ones which is a reserved instruction
    // TODO: keep track of which progmem locations are filled instead
    let all_ones = Bitvector::new(0xFFFF);
    let mut progmem = BitvectorArray::new_filled(all_ones);

    machine_check_avr::read_hex_into_progmem(&mut progmem, &hex);

    let system = machine_check_avr::ATmega328P { PROGMEM: progmem };
    machine_check::execute(system, run_args);
}
