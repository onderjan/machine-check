use std::{borrow::Cow, collections::HashMap, error::Error};

use gimli::{DebuggingInformationEntry, DwAt, Dwarf, EvaluationResult, Reader, UnitRef};
use object::{read::elf::ElfFile32, LittleEndian, Object, ObjectSection};

#[derive(Debug)]
pub enum Symbol {
    Address(u32),
    Multiple,
}

pub fn load_symbols(elf_file: &ElfFile32<LittleEndian>) -> anyhow::Result<HashMap<String, Symbol>> {
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

    let mut symbol_map = HashMap::new();

    while let Some(header) = iter.next()? {
        //eprintln!("Header: {:?}", header.offset());
        let unit = dwarf.unit(header)?;
        let unit_ref = unit.unit_ref(&dwarf);

        let mut entries = unit.entries();
        while let Some((_index, entry)) = entries.next_dfs()? {
            load_entry_symbol(&mut symbol_map, &dwarf, &unit_ref, entry)?;
        }
    }

    eprintln!("Symbols: {:#x?}", symbol_map);

    Ok(symbol_map)
}

fn load_entry_symbol<'a, R: Reader>(
    symbol_map: &mut HashMap<String, Symbol>,
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
) -> anyhow::Result<()> {
    let Some(name_attr) = entry.attr(DwAt(0x03))? else {
        return Ok(());
    };
    let Ok(s) = unit_ref.attr_string(name_attr.value()) else {
        return Ok(());
    };
    let Ok(name) = s.to_string() else {
        return Ok(());
    };

    eprintln!("Name: {:?}", name);

    if let Some(location) = entry
        .attr(DwAt(0x02))?
        .and_then(|attr| attr.exprloc_value())
    {
        let mut evaluation = location.evaluation(unit_ref.encoding());
        let EvaluationResult::RequiresIndexedAddress { index, relocate: _ } =
            evaluation.evaluate()?
        else {
            panic!("Evaluation result not RequiresIndexedAddress");
        };
        let index = dwarf.address(unit_ref, index)?;
        evaluation.resume_with_indexed_address(index)?;
        let eval = evaluation.evaluate();
        eprintln!("Eval: {:?}", eval);
        let eval_result = evaluation.result();
        eprintln!("Eval result: {:?}", eval_result);

        if eval_result.len() != 1 {
            return Ok(());
        }

        let first_piece = &eval_result[0];
        let first_piece_location = &first_piece.location;
        if let gimli::Location::Address { address } = first_piece_location {
            add_symbol(symbol_map, name, Symbol::Address(*address as u32));
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
    Ok(())
}

fn add_symbol(symbols: &mut HashMap<String, Symbol>, name: Cow<'_, str>, symbol: Symbol) {
    if symbols.insert(name.to_string(), symbol).is_some() {
        // multiple symbols with the same name
        symbols.insert(name.to_string(), Symbol::Multiple);
    }
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
