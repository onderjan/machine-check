use machine_check_avr::machine_module;
use mck::forward::ReadWrite;

fn main() {
    let hex = include_str!("basic_branch.hex");

    let reader = ihex::Reader::new(hex);

    // fill with ones which is a reserved instruction
    let unknown = ::mck::abstr::Bitvector::new_unknown();
    let mut progmem = ::mck::abstr::Array::new_filled(unknown);

    for record in reader {
        let record = match record {
            Ok(ok) => ok,
            Err(err) => panic!("Hex file read error: {}", err),
        };
        match record {
            ihex::Record::Data { offset, value } => {
                // offset is given in bytes
                if offset % 2 != 0 {
                    panic!("Unexpected noneven offset in record data");
                }
                if value.len() % 2 != 0 {
                    panic!("Unexpected noneven number of bytes in record data");
                }
                let word_offset = offset / 2;
                let mut word_index = word_offset as usize;
                for (lo, hi) in value
                    .iter()
                    .cloned()
                    .step_by(2)
                    .zip(value.iter().cloned().skip(1).step_by(2))
                {
                    // AVR has progmem words specified in little-endian order
                    let word = u16::from_le_bytes([lo, hi]);
                    progmem = progmem.write(
                        ::mck::abstr::Bitvector::new(word_index as u64),
                        ::mck::abstr::Bitvector::new(word as u64),
                    );
                    word_index += 1;
                }
            }
            ihex::Record::EndOfFile => {}
            _ => panic!("Unexpected type of record {:#?}", record),
        }
    }

    println!("Progmem: {:?}", progmem);

    let abstract_machine = machine_module::Machine {
        PROGMEM: progmem,
        dummy: ::mck::abstr::Bitvector::new(1),
    };

    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >(&abstract_machine);
}
