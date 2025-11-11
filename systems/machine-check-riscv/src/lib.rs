use anyhow::anyhow;
use clap::Args;
use machine_check::{Bitvector, BitvectorArray, ExecArgs, ExecError, ExecResult, ExecStats};
use object::{read::elf::ElfFile32, LittleEndian, Object, ObjectSection, SectionKind};

use crate::system::{machine_module, R9A02G021};

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

    //eprintln!("System: {:#?}", system);

    let input = machine_module::Input {};
    let param = machine_module::Param {};

    let state = machine_check::Machine::init(&system, &input, &param);

    let state = machine_check::Machine::next(&system, &state, &input, &param);

    eprintln!("State after first next: {:?}", state);

    Ok(machine_check::execute(system, exec_args))
}

fn parse_elf(path: &str) -> anyhow::Result<system::R9A02G021> {
    let elf_bytes = std::fs::read(path)?;
    let elf_file = ElfFile32::<LittleEndian>::parse(&*elf_bytes)?;

    for program_header in elf_file.elf_program_headers() {
        println!("{:?}", program_header);
        println!(
            "Offset: 0x{:x}, size: {:x}, flags: {:x}",
            program_header.p_offset.get(LittleEndian),
            program_header.p_memsz.get(LittleEndian),
            program_header.p_flags.get(LittleEndian),
        );
    }

    // zero is guaranteed-illegal instruction
    let zero = Bitvector::new(0);

    // program flash
    // 0x0000_0000..0x0002_0000 (17 bits,
    // store in 16-bit elements to account for compressed instructions (16-bit index)
    let mut mcu_program_flash = BitvectorArray::<16, 16>::new_filled(zero);

    for section in elf_file.sections() {
        println!(
            "{}: {:x}, {:x}, {:?}",
            section.name()?,
            section.address(),
            section.uncompressed_data()?.len(),
            section.kind()
        );

        let kind = section.kind();

        match kind {
            SectionKind::Text | SectionKind::Data | SectionKind::ReadOnlyData => {
                // just disregard the behaviour for now and load

                let mut address: u64 = section.address();
                eprintln!("Data {:x}, {:x}", address, section.size());

                match address {
                    0x0000_0000..0x0002_0000 => {
                        let data = section.uncompressed_data()?;
                        //eprintln!("Data: {:x?}", data);

                        for value in data.chunks(2) {
                            let halfword = u16::from_le_bytes(value.try_into()?);
                            mcu_program_flash[Bitvector::new(address)] =
                                Bitvector::new(halfword as u64);
                            address += 1;
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
                eprintln!("Uninitialized data")
            }

            SectionKind::Elf(number) => {
                if number == 0x70000003 {
                    eprintln!("RISC-V attribute section");
                } else {
                    return Err(anyhow!("Unsupported Elf section"));
                }
            }

            SectionKind::OtherString
            | SectionKind::Other
            | SectionKind::Debug
            | SectionKind::DebugString
            | SectionKind::Note => {
                eprintln!("Misc other {:?}", kind);
                // do nothing
            }
            SectionKind::Metadata => {
                // metadata
                eprintln!("Metadata")
            }
            _ => return Err(anyhow!("Unsupported section kind {:?}", kind)),
        }
    }

    let system = R9A02G021 {
        program_flash: mcu_program_flash,
    };

    Ok(system)
}

#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}
