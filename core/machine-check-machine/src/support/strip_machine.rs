use syn::{
    visit_mut::{self, VisitMut},
    Attribute, Meta,
};

use crate::{Description, MachineError};

pub fn strip_machine(machine: &mut Description) -> Result<(), MachineError> {
    let mut visitor = BlockVisitor {};
    for item in machine.items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    Ok(())
}

struct BlockVisitor {}
impl VisitMut for BlockVisitor {
    fn visit_local_mut(&mut self, node: &mut syn::Local) {
        strip_attributes(&mut node.attrs);
        visit_mut::visit_local_mut(self, node);
    }
    fn visit_expr_call_mut(&mut self, node: &mut syn::ExprCall) {
        strip_attributes(&mut node.attrs);
        visit_mut::visit_expr_call_mut(self, node);
    }
}

fn strip_attributes(attrs: &mut Vec<Attribute>) {
    let mut i = 0;
    while i < attrs.len() {
        let attr = &attrs[i];
        if let Meta::NameValue(meta) = &attr.meta {
            if meta.path.leading_colon.is_some()
                && meta.path.segments.len() > 2
                && &meta.path.segments[0].ident.to_string() == "mck"
                && &meta.path.segments[1].ident.to_string() == "attr"
            {
                // remove attribute
                attrs.remove(i);
            } else {
                // leave as-is, increment i
                i += 1;
            }
        }
    }
}
