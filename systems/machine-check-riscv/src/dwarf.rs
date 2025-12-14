use std::collections::HashMap;

use gimli::{
    DebuggingInformationEntry, DwAt, Dwarf, EndianSlice, EvaluationResult, Expression, Piece,
    Reader, UnitRef,
};
use object::{read::elf::ElfFile32, LittleEndian, Object, ObjectSection};

#[derive(Debug)]
pub enum Symbol {
    Unresolved,
    Address(u32),
    Multiple,
}

pub fn load_symbols(elf_file: &ElfFile32<LittleEndian>) -> anyhow::Result<HashMap<String, Symbol>> {
    let dwarf_sections = gimli::DwarfSections::load(|id| {
        elf_file
            .section_by_name(id.name())
            .map(|section| section.uncompressed_data())
            .unwrap_or(Ok(Default::default()))
    })?;
    let dwarf = dwarf_sections.borrow(|section| EndianSlice::new(section, gimli::LittleEndian));

    let mut symbol_map = HashMap::new();

    let mut unit_iter = dwarf.units();
    while let Some(header) = unit_iter.next()? {
        let unit = dwarf.unit(header)?;
        let unit_ref = unit.unit_ref(&dwarf);

        let mut entries_cursor = unit.entries();
        while let Some((_index, entry)) = entries_cursor.next_dfs()? {
            let Some(name_attr) = entry.attr(DwAt(0x03))? else {
                continue;
            };
            let Ok(s) = unit_ref.attr_string(name_attr.value()) else {
                continue;
            };
            let Ok(name) = s.to_string() else {
                continue;
            };

            eprintln!("Symbol name: {:?}", name);

            let symbol = load_entry_symbol(&dwarf, &unit_ref, entry)?;

            if symbol_map.insert(name.to_string(), symbol).is_some() {
                // multiple symbols with the same name
                symbol_map.insert(name.to_string(), Symbol::Multiple);
            }
        }
    }

    eprintln!("Symbols: {:#x?}", symbol_map);

    Ok(symbol_map)
}

fn load_entry_symbol<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
) -> anyhow::Result<Symbol> {
    eprintln!("<{:?}> {}", entry.offset().0, entry.tag());

    let mut iter = entry.attrs();
    while let Some(attr) = iter.next()? {
        eprint!("   {}: {:?}", attr.name(), attr.value());
        if let Ok(s) = unit_ref.attr_string(attr.value()) {
            eprint!(" '{}'", s.to_string_lossy()?);
        }
        eprintln!();
    }

    if let Some(location_expression) = entry
        .attr(DwAt(0x02))?
        .and_then(|attr| attr.exprloc_value())
    {
        let Some(eval_result) = evaluate_expresssion(dwarf, unit_ref, location_expression)? else {
            return Ok(Symbol::Unresolved);
        };

        if eval_result.len() != 1 {
            return Ok(Symbol::Unresolved);
        }

        let first_piece = &eval_result[0];
        let first_piece_location = &first_piece.location;
        if let gimli::Location::Address { address } = first_piece_location {
            return Ok(Symbol::Address(*address as u32));
        }
    }

    Ok(Symbol::Unresolved)
}

fn evaluate_expresssion<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    expression: Expression<R>,
) -> anyhow::Result<Option<Vec<Piece<R>>>> {
    let mut evaluation = expression.evaluation(unit_ref.encoding());

    loop {
        match evaluation.evaluate()? {
            EvaluationResult::Complete => {
                let eval_result = evaluation.result();
                eprintln!("Eval result: {:?}", eval_result);
                return Ok(Some(eval_result));
            }
            EvaluationResult::RequiresIndexedAddress { index, relocate } => {
                let index = dwarf.address(unit_ref, index)?;
                evaluation.resume_with_indexed_address(index)?;
            }
            _ => {
                // do not resolve anymore
                return Ok(None);
            }
        }
    }
}
