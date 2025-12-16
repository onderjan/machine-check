use std::{borrow::Cow, collections::HashMap, ops::Range};

use gimli::{
    AttributeValue, DebuggingInformationEntry, DwAt, Dwarf, EndianSlice, EvaluationResult, Reader,
    UnitRef,
};
use object::{read::elf::ElfFile32, LittleEndian, Object, ObjectSection};

pub struct Symbols {
    inner: HashMap<String, Symbol>,
}

#[derive(Debug)]
pub struct TypedSymbol {
    pub address: u32,
    pub byte_size: u32,
}

#[derive(Debug)]
pub enum Symbol {
    Unresolved,
    Typed(TypedSymbol),
    ProgramCounterAddress(u32),
    ProgramCounterRange(Range<u32>),
    Multiple,
}

impl Symbols {
    pub fn from_elf_file(elf_file: &ElfFile32<LittleEndian>) -> anyhow::Result<Symbols> {
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

                let symbol = load_entry_symbol(&dwarf, &unit_ref, entry)?;

                if symbol_map.insert(name.to_string(), symbol).is_some() {
                    // multiple symbols with the same name
                    symbol_map.insert(name.to_string(), Symbol::Multiple);
                }
            }
        }

        Ok(Symbols { inner: symbol_map })
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.inner.get(name)
    }

    pub fn inner(&self) -> &HashMap<String, Symbol> {
        &self.inner
    }
}

fn load_entry_symbol<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
) -> anyhow::Result<Symbol> {
    if let Some(symbol) = evaluate_typed(dwarf, unit_ref, entry)? {
        return Ok(symbol);
    }

    // DW_AT_low_pc
    if let Some(low_pc) = evaluate_attribute_address(dwarf, unit_ref, entry, 0x11, None)? {
        if let Some(high_pc) =
            evaluate_attribute_address(dwarf, unit_ref, entry, 0x12, Some(low_pc))?
        {
            return Ok(Symbol::ProgramCounterRange(Range {
                start: low_pc,
                end: high_pc,
            }));
        }

        return Ok(Symbol::ProgramCounterAddress(low_pc));
    }

    Ok(Symbol::Unresolved)
}

fn evaluate_typed<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
) -> anyhow::Result<Option<Symbol>> {
    // DW_AT_location
    let Some(location) = evaluate_attribute_address(dwarf, unit_ref, entry, 0x02, None)? else {
        return Ok(None);
    };

    let mut unit_offset = None;

    loop {
        let current_entry = if let Some(unit_offset) = unit_offset {
            let ty_entry = unit_ref.entry(unit_offset)?;
            Cow::Owned(ty_entry)
        } else {
            Cow::Borrowed(entry)
        };

        // DW_AT_byte_size
        if let Some(byte_size) = current_entry.attr(DwAt(0x0b))? {
            if let Some(byte_size) = byte_size.udata_value() {
                return Ok(Some(Symbol::Typed(TypedSymbol {
                    address: location,
                    byte_size: byte_size as u32,
                })));
            }
        };

        // DW_AT_type
        let Some(ty) = current_entry.attr(DwAt(0x49))? else {
            return Ok(None);
        };

        let AttributeValue::UnitRef(offset) = ty.value() else {
            return Ok(None);
        };

        unit_offset = Some(offset);
    }
}

fn evaluate_attribute_address<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
    attribute: u16,
    base: Option<u32>,
) -> anyhow::Result<Option<u32>> {
    let Some(attr) = entry.attr(DwAt(attribute))? else {
        return Ok(None);
    };

    if let Some(base) = base {
        if let Some(udata) = attr.udata_value() {
            return Ok(Some(base + udata as u32));
        }
    }

    if let gimli::AttributeValue::DebugAddrIndex(debug_addr_index) = attr.value() {
        let index = dwarf.address(unit_ref, debug_addr_index)?;
        return Ok(Some(index as u32));
    }

    let Some(expression) = attr.exprloc_value() else {
        return Ok(None);
    };

    let mut evaluation = expression.evaluation(unit_ref.encoding());

    loop {
        let eval_progress = evaluation.evaluate()?;

        match eval_progress {
            EvaluationResult::Complete => {
                break;
            }
            EvaluationResult::RequiresIndexedAddress { index, relocate: _ } => {
                let index = dwarf.address(unit_ref, index)?;
                evaluation.resume_with_indexed_address(index)?;
            }
            _ => {
                // do not resolve anymore
                return Ok(None);
            }
        }
    }

    let eval_result = evaluation.result();

    if eval_result.len() != 1 {
        return Ok(None);
    }

    let first_piece = &eval_result[0];
    let first_piece_location = &first_piece.location;
    let gimli::Location::Address { address } = first_piece_location else {
        return Ok(None);
    };
    Ok(Some(*address as u32))
}
