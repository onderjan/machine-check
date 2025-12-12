use std::{borrow::Cow, error::Error};

use anyhow::anyhow;
use clap::Args;
use gimli::Reader;
use machine_check::{Bitvector, BitvectorArray, ExecArgs, ExecError, ExecResult, ExecStats};
use object::{read::elf::ElfFile32, LittleEndian, Object, ObjectSection, SectionKind};

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

    /*let input = machine_module::Input {
        PIDR: BitvectorArray::new_filled(Bitvector::new(0)),
    };
    let param = machine_module::Param {};

    let mut state = machine_check::Machine::init(&system, &input, &param);

    for i in 0..1024 {
        state = machine_check::Machine::next(&system, &state, &input, &param);

        //eprintln!("Step {}: {:?}", i, state);
    }*/

    Ok(machine_check::execute(system, exec_args))
}

fn parse_elf(path: &str) -> anyhow::Result<system::R9A02G021> {
    let elf_bytes = std::fs::read(path)?;
    let elf_file = ElfFile32::<LittleEndian>::parse(&*elf_bytes)?;

    /*for program_header in elf_file.elf_program_headers() {
        eprintln!("{:?}", program_header);
        eprintln!(
            "Offset: 0x{:x}, size: {:x}, flags: {:x}",
            program_header.p_offset.get(LittleEndian),
            program_header.p_memsz.get(LittleEndian),
            program_header.p_flags.get(LittleEndian),
        );
    }*/

    load_symbols(&elf_file);

    // zero is guaranteed-illegal instruction
    let zero = Bitvector::new(0);

    // program flash
    // 0x0000_0000..0x0002_0000 (17 bits,
    // store in 16-bit elements to account for compressed instructions (16-bit index)
    let mut mcu_program_flash = BitvectorArray::<16, 16>::new_filled(zero);

    for section in elf_file.sections() {
        /*eprintln!(
            "{}: {:x}, {:x}, {:?}",
            section.name()?,
            section.address(),
            section.uncompressed_data()?.len(),
            section.kind()
        );*/

        if section
            .elf_relocation_section_index()
            .expect("Relocation section index should be gettable")
            .is_some()
        {
            return Err(anyhow!("ELF file relocations not supported"));
        }

        let kind = section.kind();

        match kind {
            SectionKind::Text | SectionKind::Data | SectionKind::ReadOnlyData => {
                // just disregard the behaviour for now and load

                let address: u64 = section.address();

                match address {
                    0x0000_0000..0x0002_0000 => {
                        let data = section.uncompressed_data()?;

                        assert_eq!(address % 2, 0);
                        let mut halfword_address = address / 2;

                        for value in data.chunks(2) {
                            let halfword = u16::from_le_bytes(value.try_into()?);
                            mcu_program_flash[Bitvector::new(halfword_address)] =
                                Bitvector::new(halfword as u64);
                            halfword_address += 1;
                        }
                    }
                    0x0101_0008..0x0101_0034 => {
                        // TODO: handle option-setting memory
                    }
                    0x2000_0000..0x2000_1000 => {
                        if section.size() > 0 {
                            panic!("Cannot copy into ECC SRAM")
                        }
                    }
                    0x2000_4000..0x2000_7000 => {
                        if section.size() > 0 {
                            panic!("Cannot copy into parity SRAM")
                        }
                    }
                    _ => {
                        unimplemented!(
                            "ELF section with address {:x}, size {:x}",
                            section.address(),
                            section.size()
                        )
                    }
                }
            }
            SectionKind::UninitializedData => {
                // this will not be loaded, but initialised by the program itself
            }

            SectionKind::Elf(number) => {
                if number == 0x70000003 {
                    // RISC-V attribute section
                } else {
                    return Err(anyhow!("Unsupported Elf section"));
                }
            }

            SectionKind::OtherString
            | SectionKind::Other
            | SectionKind::Debug
            | SectionKind::DebugString
            | SectionKind::Note => {
                // do nothing
            }
            SectionKind::Metadata => {
                // metadata
            }
            _ => return Err(anyhow!("Unsupported section kind {:?}", kind)),
        }
    }

    let system = R9A02G021 {
        program_flash: mcu_program_flash,
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

impl<'a> gimli::read::Relocate for &'a RelocationMap {
    fn relocate_address(&self, offset: usize, value: u64) -> gimli::Result<u64> {
        Ok(self.0.relocate(offset as u64, value))
    }

    fn relocate_offset(&self, offset: usize, value: usize) -> gimli::Result<usize> {
        <usize as gimli::ReaderOffset>::from_u64(self.0.relocate(offset as u64, value as u64))
    }
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

    while let Some(header) = iter.next()? {
        eprintln!("Header: {:?}", header.offset());
        dump(dwarf.unit(header)?.unit_ref(&dwarf))?;
    }

    Ok(())
}

fn dump(unit: gimli::UnitRef<impl Reader>) -> Result<(), gimli::Error> {
    // Iterate over the Debugging Information Entries (DIEs) in the unit.
    let mut entries = unit.entries();
    while let Some((index, entry)) = entries.next_dfs()? {
        eprintln!("<{:?}> {}", entry.offset().0, entry.tag());

        let mut iter = entry.attrs();

        // Iterate over the attributes in the DIE.

        loop {
            let Some(attr) = (match iter.next() {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            }) else {
                break;
            };

            eprint!("   {}: {:?}", attr.name(), attr.value());
            if let Ok(s) = unit.attr_string(attr.value()) {
                eprint!(" '{}'", s.to_string_lossy()?);
            }
            eprintln!();
        }
    }
    Ok(())
}

#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}
