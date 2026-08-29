use indexmap::IndexMap;
use syn::{Block, Stmt};

use crate::{
    context::WOuterContext,
    wir::{
        WExprProperty, WExprSubproperty, WFnSignature, WIdent, WItemFn, WProperty, WSpan,
        WSubproperty, WSubpropertyFunc, WSynBlock, WTypeId, WVisibility,
    },
    Errors,
};

impl WOuterContext {
    pub fn property_from_expr(
        mut self,
        globals: &IndexMap<WIdent, WTypeId>,
        property: WExprProperty,
    ) -> Result<WProperty, Errors> {
        let mut subproperties = Vec::new();
        let mut optional_params = globals.clone();

        for (index, subproperty) in property.subproperties.into_iter().enumerate() {
            let subproperty_ident =
                WIdent::new(format!("__mck_subproperty_{}", index), WSpan::call_site());
            optional_params.insert(subproperty_ident, self.bool_type_id());

            let subproperty = match subproperty {
                WExprSubproperty::Expr(subproperty_func) => {
                    let span = WSpan::from_syn(&subproperty_func.expr);

                    let subproperty_fn_ident = WIdent::new(format!("__mck_subfn_{}", index), span);

                    let item_fn = WItemFn {
                        visibility: WVisibility::Public(span),
                        signature: WFnSignature {
                            ident: subproperty_fn_ident,
                            inputs: vec![],
                            output: self.bool_type_id(),
                        },
                        body: WSynBlock(Block {
                            brace_token: Default::default(),
                            stmts: vec![Stmt::Expr(subproperty_func.expr, None)],
                        }),
                    };

                    let fn_id = self.add_fn(item_fn);

                    WSubproperty::Func(WSubpropertyFunc {
                        parent: subproperty_func.parent,
                        fn_id,
                        children: subproperty_func.dependencies,
                        display: subproperty_func.display,
                    })
                }
                WExprSubproperty::Next(next_operator) => WSubproperty::Next(next_operator),
                WExprSubproperty::FixedPoint(fixed_point_operator) => {
                    WSubproperty::FixedPoint(fixed_point_operator)
                }
            };

            subproperties.push(subproperty);
        }

        let ctx = self.build(&optional_params)?.infer()?.lower()?;
        let property = WProperty { ctx, subproperties };
        Ok(property)
    }
}
