use machine_check::{Bitvector, BitvectorArray};

#[doc(hidden)]
pub fn read_hex_into_progmem<const I: u32>(progmem: &mut BitvectorArray<I, 16>, hex: &str) {
    let reader = ihex::Reader::new(hex);

    let progmem_size = u64::pow(2, I);

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
                let mut word_index = word_offset as u64;
                for (lo, hi) in value
                    .iter()
                    .cloned()
                    .step_by(2)
                    .zip(value.iter().cloned().skip(1).step_by(2))
                {
                    // AVR has progmem words specified in little-endian order
                    let word = u16::from_le_bytes([lo, hi]);
                    if word_index >= progmem_size {
                        panic!("Hex file location is outside program memory");
                    }
                    let index = Bitvector::new(word_index);
                    progmem[index] = Bitvector::new(word as u64);
                    word_index += 1;
                }
            }
            ihex::Record::EndOfFile => {}
            _ => panic!("Unexpected type of record {:#?}", record),
        }
    }
}
