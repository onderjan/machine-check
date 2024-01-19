use syn::{visit_mut::VisitMut, Meta};

use crate::{MachineDescription, MachineError};

pub fn strip_machine(machine: &mut MachineDescription) -> Result<(), MachineError> {
    let mut visitor = BlockVisitor {};
    for item in machine.items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    Ok(())
}

struct BlockVisitor {}
impl VisitMut for BlockVisitor {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        let mut i = 0;
        while i < local.attrs.len() {
            let attr = &local.attrs[i];
            if let Meta::NameValue(meta) = &attr.meta {
                if meta.path.leading_colon.is_some()
                    && meta.path.segments.len() > 2
                    && &meta.path.segments[0].ident.to_string() == "mck"
                    && &meta.path.segments[1].ident.to_string() == "attr"
                {
                    // remove attribute
                    local.attrs.remove(i);
                } else {
                    // leave as-is, increment i
                    i += 1;
                }
            }
        }
    }
}
