use std::collections::BTreeMap;

use machine_check::{Bitvector, BitvectorArray};
use object::{
    read::elf::{ElfFile32, ProgramHeader},
    LittleEndian,
};

use super::dwarf;

use crate::{
    dwarf::{Symbol, Symbols},
    system::System,
};

/// Parses an ELF file and returns the system and usable symbols.
pub fn parse_elf(path: &str) -> anyhow::Result<(System, dwarf::Symbols)> {
    // zero is guaranteed-illegal instruction
    let halfword_zero = Bitvector::<16>::new(0);

    // program flash
    // 0x0000_0000..0x0002_0000 (17 bits)
    // store in 16-bit elements to account for compressed instructions (16-bit index)
    let mut program_flash = BitvectorArray::<16, 16>::new_filled(halfword_zero);

    // load the program flash from program headers
    let elf_bytes = std::fs::read(path)?;
    let elf_file = ElfFile32::<LittleEndian>::parse(&*elf_bytes)?;

    for program_header in elf_file.elf_program_headers() {
        let header_type = program_header.p_type.get(LittleEndian);

        if header_type != 0x01 {
            // not LOAD
            continue;
        }

        let address = program_header.p_paddr.get(LittleEndian);
        let size = program_header.p_memsz.get(LittleEndian);

        let data = program_header
            .data(LittleEndian, &*elf_bytes)
            .expect("Program header data should be present");

        match address {
            0x0000_0000..0x0002_0000 => {
                // load into program flash

                let mut data_iter = data.iter().copied();

                let mut address = address;

                // handle non-aligned first byte
                if address % 2 == 1 {
                    if let Some(byte) = data_iter.next() {
                        // first byte, it will be high byte
                        program_flash[Bitvector::new((address / 2) as u64)] = program_flash
                            [Bitvector::new((address / 2) as u64)]
                            & Bitvector::<16>::new(0x00FF)
                            | (Bitvector::<16>::new(byte as u64) << Bitvector::<16>::new(8));
                        address += 1;
                    }
                }

                // handle the rest
                for value in data.chunks(2) {
                    if value.len() == 2 {
                        // full halfword, replace it
                        let halfword = u16::from_le_bytes(value.try_into()?);
                        program_flash[Bitvector::new((address / 2) as u64)] =
                            Bitvector::new(halfword as u64);
                    } else {
                        // last byte, it will be low byte
                        let byte = value[0];
                        program_flash[Bitvector::new((address / 2) as u64)] = program_flash
                            [Bitvector::new((address / 2) as u64)]
                            & Bitvector::<16>::new(0xFF00)
                            | (Bitvector::<16>::new(byte as u64));
                    }

                    address += value.len() as u32;
                }
            }
            0x0101_0008..0x0101_0034 => {
                // the option-setting memory cannot be read or written to
                // within the system, do not load
            }
            0x2000_0000..0x2000_1000 => {
                // ECC SRAM, not persistent, do not load
            }
            0x2000_4000..0x2000_7000 => {
                // parity SRAM, not persistent, do not load
            }
            _ => {
                unimplemented!(
                    "ELF program header with offset {:x}, size {:x}",
                    address,
                    size
                )
            }
        }
    }

    // load the symbols
    let symbols = Symbols::from_elf_file(&elf_file)?;

    if log::log_enabled!(log::Level::Debug) {
        // debug-log the symbols
        // sort alphabetically and filter out unusable symbols
        let mut usable = BTreeMap::new();

        for (name, symbol) in symbols.inner().iter() {
            if !matches!(symbol, Symbol::Unresolved | Symbol::Multiple) {
                usable.insert(name, symbol);
            }
        }

        log::debug!("Usable symbols: {:#X?}", usable);
    }

    let system = System { program_flash };

    Ok((system, symbols))
}
