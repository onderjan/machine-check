use machine_check::{Bitvector, BitvectorArray};

fn main() {
    // TODO: rework argument parsing for systems to mesh well with machine-check
    let args: Vec<String> = std::env::args().collect();

    let mut processed_args = Vec::new();
    let mut first = true;
    let mut next_arg_hex_file = false;
    let mut hex_file = None;
    for arg in args.into_iter() {
        if first {
            // do not process first parameter
            processed_args.push(arg);
            first = false;
            continue;
        }
        if next_arg_hex_file {
            hex_file = Some(arg);
            next_arg_hex_file = false;
        } else if arg == "--hex-file" {
            next_arg_hex_file = true;
        } else {
            processed_args.push(arg);
        }
    }

    let Some(hex_file) = hex_file else {
        eprintln!("No hex file parameter pair, specify it");
        return;
    };

    let hex = match std::fs::read_to_string(hex_file) {
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
    machine_check::run_with_args(system, processed_args.into_iter());
}
