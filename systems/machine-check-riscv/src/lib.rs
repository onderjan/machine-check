use std::{borrow::Cow, collections::HashMap, error::Error};

use clap::Args;
use gimli::{DwAt, EvaluationResult, Reader};
use machine_check::{Bitvector, BitvectorArray, ExecArgs, ExecError, ExecResult, ExecStats};
use object::{
    read::elf::{ElfFile32, ProgramHeader},
    LittleEndian, Object, ObjectSection,
};

use crate::system::R9A02G021;

mod system;

pub fn execute(args: impl Iterator<Item = String>) -> ExecResult {
    let (exec_args, system_args) = machine_check::parse_args::<SystemArgs>(args);
    match execute_with_args(exec_args, system_args) {
        Ok(ok) => ok,
        Err(err) => {
            let err = err.to_string();
            eprintln!("{}", err);
            ExecResult {
                result: Err(ExecError::OtherError(err)),
                stats: ExecStats::default(),
            }
        }
    }
}

pub fn execute_with_args(
    exec_args: ExecArgs,
    system_args: SystemArgs,
) -> anyhow::Result<ExecResult> {
    let system = parse_elf(&system_args.elf_file)?;

    /*let input = system::machine_module::Input {
        PIDR: BitvectorArray::new_filled(Bitvector::new(0)),
    };
    let param = system::machine_module::Param {};

    let mut state = machine_check::Machine::init(&system, &input, &param);

    for i in 0..1024 {
        state = machine_check::Machine::next(&system, &state, &input, &param);

        eprintln!("Step {}: {:#X?}", i, state);
    }*/

    Ok(machine_check::execute(system, exec_args))
}

fn parse_elf(path: &str) -> anyhow::Result<system::R9A02G021> {
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

    load_symbols(&elf_file)?;

    eprintln!("Program flash: {:#X?}", program_flash);
    eprintln!("Initial SRAM: {:#X?}", initial_sram_parity);

    let system = R9A02G021 {
        program_flash,
        initial_sram_parity,
    };

    Ok(system)
}

#[derive(Debug, Default)]
struct Section<'data> {
    pub data: Cow<'data, [u8]>,
    pub relocations: RelocationMap,
}

#[derive(Debug, Default)]
struct RelocationMap(object::read::RelocationMap);

impl gimli::read::Relocate for &'_ RelocationMap {
    fn relocate_address(&self, offset: usize, value: u64) -> gimli::Result<u64> {
        Ok(self.0.relocate(offset as u64, value))
    }

    fn relocate_offset(&self, offset: usize, value: usize) -> gimli::Result<usize> {
        <usize as gimli::ReaderOffset>::from_u64(self.0.relocate(offset as u64, value as u64))
    }
}

enum Symbol {
    Address(u32),
    Multiple,
}

fn load_symbols(elf_file: &ElfFile32<LittleEndian>) -> anyhow::Result<()> {
    fn load_section<'data>(
        object: &ElfFile32<'data, LittleEndian>,
        name: &str,
    ) -> Result<Section<'data>, Box<dyn Error>> {
        Ok(match object.section_by_name(name) {
            Some(section) => Section {
                data: section.uncompressed_data()?,
                relocations: RelocationMap(section.relocation_map()?),
            },
            None => Default::default(),
        })
    }

    fn borrow_section<'data>(section: &'data Section<'data>) -> impl Reader + use<'data> {
        let slice =
            gimli::EndianSlice::new(std::borrow::Cow::as_ref(&section.data), gimli::LittleEndian);
        gimli::RelocateReader::new(slice, &section.relocations)
    }

    let dwarf_sections = gimli::DwarfSections::load(|id| load_section(elf_file, id.name()))
        .expect("Should read DWARF sections");

    let dwarf = dwarf_sections.borrow(|section| borrow_section(section));

    let mut iter = dwarf.units();

    let mut symbols = HashMap::new();

    while let Some(header) = iter.next()? {
        //eprintln!("Header: {:?}", header.offset());
        let unit = dwarf.unit(header)?;
        let unit_ref = unit.unit_ref(&dwarf);

        let mut entries = unit.entries();
        while let Some((_index, entry)) = entries.next_dfs()? {
            let Some(name_attr) = entry.attr(DwAt(0x03))? else {
                continue;
            };
            let Ok(s) = unit_ref.attr_string(name_attr.value()) else {
                continue;
            };
            let Ok(name) = s.to_string() else {
                continue;
            };

            eprintln!("Name: {:?}", name);

            if let Some(location) = entry
                .attr(DwAt(0x02))?
                .and_then(|attr| attr.exprloc_value())
            {
                let mut evaluation = location.evaluation(unit.encoding());
                let index = 0;
                let EvaluationResult::RequiresIndexedAddress { index, relocate } =
                    evaluation.evaluate()?
                else {
                    panic!("Evaluation result not RequiresIndexedAddress");
                };
                let index = dwarf.address(&unit_ref, index)?;
                evaluation.resume_with_indexed_address(index)?;
                let eval = evaluation.evaluate();
                eprintln!("Eval: {:?}", eval);
                let eval_result = evaluation.result();
                eprintln!("Eval result: {:?}", eval_result);

                if eval_result.len() != 1 {
                    continue;
                }

                let first_piece = &eval_result[0];
                let first_piece_location = &first_piece.location;
                if let gimli::Location::Address { address } = first_piece_location {
                    symbols.insert(name.to_string(), *address);
                }
            }

            eprintln!("<{:?}> {}", entry.offset().0, entry.tag());

            let mut iter = entry.attrs();
            while let Some(attr) = iter.next()? {
                eprint!("   {}: {:?}", attr.name(), attr.value());
                if let Ok(s) = unit_ref.attr_string(attr.value()) {
                    eprint!(" '{}'", s.to_string_lossy()?);
                }
                eprintln!();
            }
        }

        eprintln!("Symbols: {:#x?}", symbols);
    }

    Ok(())
}

#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}
