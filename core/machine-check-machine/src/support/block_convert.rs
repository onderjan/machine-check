use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Block, Ident, Item, Stmt, Type};

use crate::{util::create_let_bare, MachineError};

pub fn block_convert(
    items: &mut [Item],
    temporary_manager: &mut TemporaryManager,
    conversion_fn: fn(&mut TemporaryManager, &mut Block) -> Result<(), MachineError>,
) -> Result<(), MachineError> {
    // convert to three-address code by adding temporaries
    let mut visitor = Visitor {
        temporary_manager,
        conversion_fn,
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    std::mem::replace(&mut temporary_manager.result, Ok(()))
}

pub struct TemporaryManager {
    inside_block: bool,
    next_temp_counter: u64,
    created_temporaries: Vec<(Ident, Option<Type>)>,
    result: Result<(), MachineError>,
}

impl TemporaryManager {
    pub fn new() -> Self {
        Self {
            inside_block: false,
            next_temp_counter: 0,
            created_temporaries: Vec::new(),
            result: Ok(()),
        }
    }

    pub fn create_temporary_ident(&mut self, span: Span, ty: Option<Type>) -> Ident {
        if !self.inside_block {
            panic!("Temporary ident cannot be created outside a block");
        }
        let tmp_ident = Ident::new(
            format!("__mck_tmp_{}", self.next_temp_counter).as_str(),
            span,
        );
        self.created_temporaries.push((tmp_ident.clone(), ty));

        self.next_temp_counter = self
            .next_temp_counter
            .checked_add(1)
            .expect("Temp counter should not overflow");
        tmp_ident
    }
}

struct Visitor<'a> {
    temporary_manager: &'a mut TemporaryManager,
    conversion_fn: fn(&mut TemporaryManager, &mut Block) -> Result<(), MachineError>,
}
impl VisitMut for Visitor<'_> {
    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        // call the conversion function
        self.temporary_manager.inside_block = true;
        let conversion_result =
            (self.conversion_fn)(self.temporary_manager, &mut impl_item_fn.block);
        self.temporary_manager.inside_block = false;

        // prefer the first error
        if self.temporary_manager.result.is_ok() {
            self.temporary_manager.result = conversion_result;
        }

        // prefix the function block with newly created temporaries
        // do not add types to temporaries, they will be inferred later
        let mut stmts: Vec<Stmt> = self
            .temporary_manager
            .created_temporaries
            .drain(..)
            .map(|(tmp_ident, tmp_type)| create_let_bare(tmp_ident, tmp_type))
            .collect();
        stmts.append(&mut impl_item_fn.block.stmts);
        impl_item_fn.block.stmts.append(&mut stmts);
    }
}
