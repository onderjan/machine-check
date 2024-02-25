use syn::Item;

use crate::{
    support::block_converter::{convert_block, TemporaryManager},
    MachineError,
};

mod finish;
mod process;

pub fn convert_to_tac(
    items: &mut [Item],
    temporary_manager: &mut TemporaryManager,
) -> Result<(), MachineError> {
    convert_block(items, temporary_manager, |temporary_manager, block| {
        let mut converter = Converter { temporary_manager };
        let process_result = converter.process_block(block);
        let finish_result = converter.finish_block(block);
        process_result.and(finish_result)
    })

    /*
    // convert to three-address code by adding temporaries
    let mut visitor = Visitor {
        result: Ok(()),
        next_temp_counter: 0,
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    visitor.result*/
}

struct Converter<'a> {
    temporary_manager: &'a mut TemporaryManager,
}
