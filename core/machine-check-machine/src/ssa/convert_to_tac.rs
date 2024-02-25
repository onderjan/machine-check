use syn::{visit_mut::VisitMut, Ident, Item, Stmt};

use crate::{util::create_let_bare, MachineError};

mod finish;
mod process;

pub fn convert_to_tac(items: &mut [Item]) -> Result<(), MachineError> {
    // convert to three-address code by adding temporaries
    let mut visitor = Visitor {
        result: Ok(()),
        next_temp_counter: 0,
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    visitor.result
}

struct Visitor {
    next_temp_counter: u64,
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor {
    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        let result = self.process_impl_item_fn(impl_item_fn);
        if let Err(err) = result {
            self.result = Err(err);
        }
    }
}

impl Visitor {
    fn process_impl_item_fn(
        &mut self,
        impl_item_fn: &mut syn::ImplItemFn,
    ) -> Result<(), MachineError> {
        let mut converter = Converter {
            next_temp_counter: &mut self.next_temp_counter,
            created_temporaries: vec![],
        };
        converter.process_block(&mut impl_item_fn.block)?;
        converter.finish_block(&mut impl_item_fn.block)?;

        // prefix the function block with newly created temporaries
        // do not add types to temporaries, they will be inferred later
        let mut stmts: Vec<Stmt> = converter
            .created_temporaries
            .iter()
            .map(|tmp_ident| create_let_bare(tmp_ident.clone(), None))
            .collect();
        stmts.append(&mut impl_item_fn.block.stmts);
        impl_item_fn.block.stmts.append(&mut stmts);

        Ok(())
    }
}

struct Converter<'a> {
    next_temp_counter: &'a mut u64,
    created_temporaries: Vec<Ident>,
}

impl Converter<'_> {
    fn get_and_increment_temp_counter(&mut self) -> u64 {
        let result = *self.next_temp_counter;
        *self.next_temp_counter = self
            .next_temp_counter
            .checked_add(1)
            .expect("Temp counter should not overflow");
        result
    }
}
