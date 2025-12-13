use machine_check::{Bitvector, BitvectorArray};
use object::{
    read::elf::{ElfFile32, ProgramHeader},
    LittleEndian,
};

use super::dwarf;

use crate::system::R9A02G021;

pub fn parse_elf(path: &str) -> anyhow::Result<R9A02G021> {
    // zero is guaranteed-illegal instruction
    let halfword_zero = Bitvector::<16>::new(0);
    let byte_zero = Bitvector::<8>::new(0);

    // program flash
    // 0x0000_0000..0x0002_0000 (17 bits,
    // store in 16-bit elements to account for compressed instructions (16-bit index)
    let mut program_flash = BitvectorArray::<16, 16>::new_filled(halfword_zero);

    // SRAM (parity)
    let mut initial_sram_parity = BitvectorArray::<14, 8>::new_filled(byte_zero);

    // get from program headers

    let elf_bytes = std::fs::read(path)?;
    let elf_file = ElfFile32::<LittleEndian>::parse(&*elf_bytes)?;

    for program_header in elf_file.elf_program_headers() {
        eprintln!("{:?}", program_header);

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
                eprintln!(
                    "Loading program flash {:#X?}..{:#X?}",
                    address,
                    address + size
                );

                let mut data_iter = data.iter().copied();

                let mut address = address;

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

                for value in data.chunks(2) {
                    if value.len() == 2 {
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
                // TODO: handle option-setting memory
            }
            0x2000_0000..0x2000_1000 => {
                panic!("Cannot copy into ECC SRAM")
            }
            0x2000_4000..0x2000_7000 => {
                let mut relative_address = (address - 0x2000_4000) as u64;

                for value in data.iter().cloned() {
                    initial_sram_parity[Bitvector::new(relative_address)] =
                        Bitvector::new(value as u64);
                    relative_address += 1;
                }
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

    dwarf::load_symbols(&elf_file)?;

    eprintln!("Program flash: {:#X?}", program_flash);
    eprintln!("Initial SRAM: {:#X?}", initial_sram_parity);

    let system = R9A02G021 {
        program_flash,
        initial_sram_parity,
    };

    Ok(system)
}
