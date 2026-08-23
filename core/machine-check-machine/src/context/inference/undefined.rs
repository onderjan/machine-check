/*use crate::{
    into_wir::Error,
    wir::{WBlock, WFnId, WIdent, WTypeId, YTac},
};

impl super::WInferenceContext {
    pub fn convert_fn_undefined_to_args(
        &mut self,
        fn_id: WFnId,
        //ident_ty_fn: impl Fn(WIdent) -> Result<WTypeId, Error>,
    ) -> Result<Vec<WIdent>, Error> {
        let func = self.definitions.function_by_id_mut(fn_id);
        func.body.locals.
        let undefined = extract_undefined_from_block(&func.body.block)?;

        Ok(undefined)
    }
}

pub fn extract_undefined_from_block(block: &WBlock<YTac>) -> Result<Vec<WIdent>, Error> {
    for stmt in block.stmts {

    }
}
*/
