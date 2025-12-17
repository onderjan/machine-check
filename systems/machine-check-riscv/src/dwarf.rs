use std::{borrow::Cow, collections::HashMap, ops::Range};

use gimli::{
    AttributeValue, DebuggingInformationEntry, DwAt, Dwarf, EndianSlice, EvaluationResult, Reader,
    UnitRef,
};
use object::{read::elf::ElfFile32, LittleEndian, Object, ObjectSection};

/// A map of symbols.
pub struct Symbols {
    inner: HashMap<String, Symbol>,
}

/// A symbol that contains an address and byte size.
#[derive(Debug)]
pub struct TypedSymbol {
    pub address: u32,
    pub byte_size: u32,
}

/// A debug symbol.
#[derive(Debug)]
pub enum Symbol {
    Unresolved,
    Typed(TypedSymbol),
    ProgramCounterAddress(u32),
    ProgramCounterRange(Range<u32>),
    Multiple,
}

impl Symbols {
    /// Produces symbols from an ELF file.
    pub fn from_elf_file(elf_file: &ElfFile32<LittleEndian>) -> anyhow::Result<Symbols> {
        // get the DWARF from the ELF file
        let dwarf_sections = gimli::DwarfSections::load(|id| {
            elf_file
                .section_by_name(id.name())
                .map(|section| section.uncompressed_data())
                .unwrap_or(Ok(Default::default()))
        })?;
        let dwarf = dwarf_sections.borrow(|section| EndianSlice::new(section, gimli::LittleEndian));

        let mut symbol_map = HashMap::new();

        let mut unit_iter = dwarf.units();
        // iterate over DWARF units
        while let Some(header) = unit_iter.next()? {
            let unit = dwarf.unit(header)?;
            let unit_ref = unit.unit_ref(&dwarf);

            // iterate over unit entries
            let mut entries_cursor = unit.entries();
            while let Some((_index, entry)) = entries_cursor.next_dfs()? {
                // skip entries that do not have a string name
                let Some(name_attr) = entry.attr(DwAt(0x03))? else {
                    continue;
                };
                let Ok(s) = unit_ref.attr_string(name_attr.value()) else {
                    continue;
                };
                let Ok(name) = s.to_string() else {
                    continue;
                };

                // process the symbol
                let symbol = entry_symbol(&dwarf, &unit_ref, entry)?;

                // try to insert the symbols
                if symbol_map.insert(name.to_string(), symbol).is_some() {
                    // insert that there are multiple symbols with the same name
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

fn entry_symbol<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
) -> anyhow::Result<Symbol> {
    // see if it is typed first
    if let Some(symbol) = evaluate_typed(dwarf, unit_ref, entry)? {
        return Ok(symbol);
    }

    // try to see if it has a program counter address/range
    // DW_AT_low_pc
    if let AddressEval::Address(low_pc) =
        evaluate_attribute_address(dwarf, unit_ref, entry, 0x11, None)?
    {
        // DW_AT_high_pc
        match evaluate_attribute_address(dwarf, unit_ref, entry, 0x12, Some(low_pc))? {
            AddressEval::Absent => {
                // PC address, definitely only has low_pc and not high_pc
                Ok(Symbol::ProgramCounterAddress(low_pc))
            }
            AddressEval::EvalFailed => {
                // PC range, but evaluation failed
                Ok(Symbol::Unresolved)
            }
            AddressEval::Address(high_pc) => {
                // PC range
                Ok(Symbol::ProgramCounterRange(Range {
                    start: low_pc,
                    end: high_pc,
                }))
            }
        }
    } else {
        Ok(Symbol::Unresolved)
    }
}

fn evaluate_typed<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
) -> anyhow::Result<Option<Symbol>> {
    // try to get an address from DW_AT_location
    let location = match evaluate_attribute_address(dwarf, unit_ref, entry, 0x02, None)? {
        AddressEval::Absent => return Ok(None),
        AddressEval::EvalFailed => return Ok(Some(Symbol::Unresolved)),
        AddressEval::Address(location) => location,
    };

    // it may be necessary to go into types recursively before finding the true base
    // store the unit offset; if none, the argument unit_ref will be used
    let mut unit_offset = None;

    loop {
        let current_entry = if let Some(unit_offset) = unit_offset {
            let ty_entry = unit_ref.entry(unit_offset)?;
            Cow::Owned(ty_entry)
        } else {
            Cow::Borrowed(entry)
        };

        // try to get DW_AT_byte_size
        if let Some(byte_size) = current_entry.attr(DwAt(0x0b))? {
            if let Some(byte_size) = byte_size.udata_value() {
                // we have found a valid byte size, create the typed symbol
                return Ok(Some(Symbol::Typed(TypedSymbol {
                    address: location,
                    byte_size: byte_size as u32,
                })));
            }
        };

        // look at whether this has a DW_AT_type
        let Some(ty) = current_entry.attr(DwAt(0x49))? else {
            // we have an address, but we have not found a byte size
            return Ok(Some(Symbol::Unresolved));
        };

        let AttributeValue::UnitRef(offset) = ty.value() else {
            // we have an address, but we have not found a byte size
            return Ok(Some(Symbol::Unresolved));
        };

        unit_offset = Some(offset);
    }
}

enum AddressEval {
    Absent,
    EvalFailed,
    Address(u32),
}

fn evaluate_attribute_address<'a, R: Reader>(
    dwarf: &'a Dwarf<R>,
    unit_ref: &UnitRef<'a, R>,
    entry: &DebuggingInformationEntry<'a, 'a, R>,
    attribute: u16,
    base: Option<u32>,
) -> anyhow::Result<AddressEval> {
    let Some(attr) = entry.attr(DwAt(attribute))? else {
        // the attribute is not present here
        return Ok(AddressEval::Absent);
    };

    if let Some(base) = base {
        if let Some(udata) = attr.udata_value() {
            // the udata gives an offset from the base
            return Ok(AddressEval::Address(base + udata as u32));
        }
    }

    if let gimli::AttributeValue::DebugAddrIndex(debug_addr_index) = attr.value() {
        // we immediately obtain an address
        let index = dwarf.address(unit_ref, debug_addr_index)?;
        return Ok(AddressEval::Address(index as u32));
    }

    let Some(expression) = attr.exprloc_value() else {
        // attribute is present, but we could not figure out the address
        return Ok(AddressEval::EvalFailed);
    };

    // evaluate the DWARF expression
    let mut evaluation = expression.evaluation(unit_ref.encoding());

    loop {
        let eval_progress = evaluation.evaluate()?;

        match eval_progress {
            EvaluationResult::Complete => {
                // evaluation is complete
                break;
            }
            EvaluationResult::RequiresIndexedAddress { index, relocate: _ } => {
                // give the indexed address and continue
                let index = dwarf.address(unit_ref, index)?;
                evaluation.resume_with_indexed_address(index)?;
            }
            _ => {
                // we could not figure out the address
                return Ok(AddressEval::EvalFailed);
            }
        }
    }

    let eval_result = evaluation.result();

    if eval_result.len() != 1 {
        // not a single piece of DWARF evaluation result
        return Ok(AddressEval::EvalFailed);
    }

    let first_piece = &eval_result[0];
    let first_piece_location = &first_piece.location;
    let gimli::Location::Address { address } = first_piece_location else {
        // the piece is not an address
        return Ok(AddressEval::EvalFailed);
    };
    Ok(AddressEval::Address(*address as u32))
}
