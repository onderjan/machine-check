use std::collections::BTreeMap;

use machine_check_common::ir_common::IrStdBinaryOp;
use syn::{Type, TypePath};

use crate::{
    context::{bitvector_type, bool_type, signed_type, unsigned_type},
    into_wir::{Error, ErrorType},
    wir::{
        WBlock, WCall, WCallArg, WExpr, WExprHighCall, WIdent, WIndexedExpr, WIndexedIdent,
        WMacroableStmt, WPartialPathArgument, WPartialPathGenerics, WPartialType, 
        WTypeId, YTac,
    },
};

impl super::WInferenceContext {
    pub(super) fn add_block_constraints(
        &mut self,
        types: &BTreeMap<WIdent, WTypeId>,
        block: &WBlock<YTac>,
    ) -> Result<(), Error> {
        eprintln!("Adding constraints for block {:?}", block);
        for stmt in &block.stmts {
            eprintln!("Should add constraints for statement {:#?}", stmt);
            match stmt {
                WMacroableStmt::Assign(assign) => {
                    let left = match &assign.left {
                        WIndexedIdent::Indexed(_wident, _wident1) => {
                            todo!("Constraints for indexed left")
                        }
                        WIndexedIdent::NonIndexed(ident) => ident,
                    };
                    let right = match &assign.right {
                        WIndexedExpr::Indexed(_base_expr, _ident) => {
                            todo!("Constraints for indexed right")
                        }
                        WIndexedExpr::NonIndexed(expr) => expr,
                    };
                    eprintln!(
                        "Should add constraints for left {:?}, right {:?}",
                        left, right
                    );

                    let left_ty = get_type(types, left)?;

                    match right {
                        WExpr::Move(right) => {
                            let right_ty = get_type(types, right)?;
                            self.add_eq_constraint(left_ty, right_ty);
                        }
                        WExpr::Call(call) => {
                            self.add_call_constraint(types, left_ty, call)?;
                        }
                        WExpr::Field(expr_field) => {
                            let base_ty = get_type(types, &expr_field.base)?;
                            // TODO: this should be in fixpoint to work with inference well
                            let mut base_ty = &self.types[base_ty.index()];

                            eprintln!("Field {:?}: base type {:?}", expr_field, base_ty);

                            while let WPartialType::Reference(ty) = base_ty {
                                base_ty = ty.as_ref();
                            }

                            if let WPartialType::Path(base_path) = base_ty {
                                eprintln!("Field {:?}: base path {:?}", expr_field, base_path);
                                let base_path = base_path.clone().without_generics();
                                let base_def = self.definitions.datatype(&base_path);
                                eprintln!("Base def: {:?}", base_def);
                                if let Some(base_def) = base_def {
                                    if let Some(field) = base_def.def.fields.get(&expr_field.member)
                                    {
                                        // TODO: dereference
                                        eprintln!("Member type: {:?}", field);
                                        self.add_eq_constraint(left_ty.clone(), field.ty.clone());
                                    }
                                }
                            }
                        }
                        WExpr::Struct(expr_struct) => {
                            self.add_eq_constraint(left_ty, expr_struct.ty.clone());
                        }
                        WExpr::Reference(_wexpr_reference) => {
                            // TODO: process references
                        }
                        WExpr::Lit(_lit, _) => {
                            // TODO: process literals
                            /*if is_property {
                                // ignore
                            } else {
                                todo!("Literal")
                            }*/
                        }
                    }
                }
                WMacroableStmt::If(stmt_if) => {
                    eprintln!("Should add constraints for if {:#?}", stmt_if);
                    self.add_block_constraints(types, &stmt_if.then_block)?;
                    self.add_block_constraints(types, &stmt_if.else_block)?;
                }
                WMacroableStmt::PanicMacro(_stmt_panic_macro) => {
                    // panic macro returns a never type
                    // TODO: never type
                }
            }
        }
        Ok(())
    }

    fn add_call_constraint(
        &mut self,
        types: &BTreeMap<WIdent, WTypeId>,
        left_ty: WTypeId,
        call: &WExprHighCall,
    ) -> Result<(), Error> {
        eprintln!("Call");
        match call {
            WExprHighCall::Call(call) => {
                return self.add_normal_call_constraint(types, left_ty, call);
            }
            WExprHighCall::StdUnary(unary) => {
                // both not and neg return the same type as the operand
                let operand_ty = get_type(types, &unary.operand)?;
                self.add_eq_constraint(left_ty, operand_ty);
            }
            WExprHighCall::StdBinary(binary) => {
                let a_ty = get_type(types, &binary.a)?;
                let b_ty = get_type(types, &binary.b)?;

                match binary.op {
                    IrStdBinaryOp::Eq
                    | IrStdBinaryOp::Ne
                    | IrStdBinaryOp::Lt
                    | IrStdBinaryOp::Le
                    | IrStdBinaryOp::Gt
                    | IrStdBinaryOp::Ge => {
                        // constrain the inputs to be of the same type
                        self.add_eq_constraint(a_ty, b_ty);
                        // constrain the output to be a Boolean
                        let bool_ty = self.partial_type_id(bool_type());
                        self.add_eq_constraint(left_ty, bool_ty);
                    }
                    IrStdBinaryOp::BitAnd
                    | IrStdBinaryOp::BitOr
                    | IrStdBinaryOp::BitXor
                    | IrStdBinaryOp::Shl
                    | IrStdBinaryOp::Shr
                    | IrStdBinaryOp::Add
                    | IrStdBinaryOp::Sub
                    | IrStdBinaryOp::Mul
                    | IrStdBinaryOp::Div
                    | IrStdBinaryOp::Rem => {
                        // constrain both inputs to output
                        self.add_eq_constraint(left_ty.clone(), a_ty);
                        self.add_eq_constraint(left_ty, b_ty);
                    }
                }
            }
        }
        Ok(())
    }

    fn add_normal_call_constraint(
        &mut self,
        types: &BTreeMap<WIdent, WTypeId>,
        left_ty: WTypeId,
        call: &WCall,
    ) -> Result<(), Error> {
        if call.fn_path.leading_colon.is_some() && call.fn_path.segments.len() == 3 {
            let segments = &call.fn_path.segments;
            let is_bitvector = segments[1].ident.name() == "Bitvector";
            let is_unsigned = segments[1].ident.name() == "Unsigned";
            let is_signed = segments[1].ident.name() == "Signed";

            eprintln!(
                "Bitvector: {}, unsigned: {}, signed: {}",
                is_bitvector, is_unsigned, is_signed
            );

            if segments[0].ident.name() == "machine_check"
                && (is_bitvector || is_unsigned || is_signed)
                && segments[2].ident.name() == "new"
            {
                let mut width = None;
                if let Some(generics) = &segments[1].generics {
                    if generics.arguments.len() == 1 {
                        if let WPartialPathArgument::Uint(width_arg, _span) = generics.arguments[0]
                        {
                            width = Some(width_arg)
                        }
                    }
                }
                let ty = if is_bitvector {
                    bitvector_type(width)
                } else if is_unsigned {
                    unsigned_type(width)
                } else {
                    signed_type(width)
                };

                // constrain the output to be a bitvector of the given width
                let bitvector_ty = self.partial_type_id(ty);
                self.add_eq_constraint(left_ty.clone(), bitvector_ty);

                return Ok(());
            }

            if segments[0].ident.name() == "machine_check"
                && segments[1].ident.name() == "Ext"
                && segments[2].ident.name() == "ext"
            {
                let mut width = None;
                if let Some(generics) = &segments[1].generics {
                    if generics.arguments.len() == 1 {
                        if let WPartialPathArgument::Uint(width_arg, span) = generics.arguments[0] {
                            width = Some((width_arg, span))
                        }
                    }
                }
                let span = call.fn_path.span();
                if call.args.len() != 1 {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Expected exactly 1 argument")),
                        span,
                    ));
                }
                let WCallArg::Ident(ident) = &call.args[0] else {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Expected ident in argument")),
                        span,
                    ));
                };
                let right_ty = get_type(types, ident)?;

                // add constraints

                let right_ty = &self.types[right_ty.index()];

                eprintln!("Should add ext constraints, right type: {:?}", right_ty);
                match right_ty {
                    WPartialType::Path(path) => {
                        if path.matches_absolute(&["machine_check", "Bitvector"])
                            || path.matches_absolute(&["machine_check", "Signed"])
                            || path.matches_absolute(&["machine_check", "Unsigned"])
                        {
                            // remake the generics
                            let mut path = path.clone();

                            path.segments[1].generics = if let Some((width_arg, span)) = width {
                                Some(WPartialPathGenerics {
                                    turbofish: None,
                                    arguments: vec![WPartialPathArgument::Uint(width_arg, span)],
                                })
                            } else {
                                None
                            };

                            let constraint_ty = self.type_id(&Type::Path(TypePath {
                                qself: None,
                                path: path.into_syn(),
                            }))?;
                            self.add_eq_constraint(left_ty.clone(), constraint_ty);
                        }
                    }
                    WPartialType::Reference(_reference) => {
                        // todo: ext reference
                    }
                    WPartialType::Infer(_) => {
                        // todo: infer reference
                    }
                }

                return Ok(());
            }
        }

        if call.fn_path.leading_colon.is_some() && call.fn_path.segments.len() == 4 {
            let segments = &call.fn_path.segments;
            if segments[0].ident.name() == "std"
                && segments[1].ident.name() == "convert"
                && segments[2].ident.name() == "Into"
                && segments[3].ident.name() == "into"
            {
                eprintln!("Processing Into");
                let span = call.fn_path.span();
                let WCallArg::Ident(_ident) = &call.args[0] else {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Expected ident in argument")),
                        span,
                    ));
                };
                // TODO: check whether the Into conversion is permitted
                //let right_ty = get_type(types, ident)?;

                if let Some(generics) = &segments[2].generics {
                    if generics.arguments.len() == 1 {
                        if let WPartialPathArgument::Type(into_ty) = &generics.arguments[0] {
                            // add constraint for the left type
                            let into_ty = self.partial_type_id(into_ty.clone());
                            eprintln!("Adding Into constraint: {:?} == {:?}", left_ty, into_ty);
                            self.add_eq_constraint(left_ty, into_ty);
                        }
                    }
                }

                return Ok(());
            }
        }

        let call_path = call.fn_path.clone().without_generics();

        if let Some(fn_def) = self.definitions.function_by_path(call_path.clone()) {
            let signature = &fn_def.signature;

            let num_expected = signature.inputs.len();
            let num_provided = call.args.len();

            if num_expected != num_provided {
                return Err(Error::new(
                    ErrorType::WrongNumberOfArguments(num_expected, num_provided),
                    call_path.span(),
                ));
            }

            let output_ty = signature.output.clone();

            // constrain left with output
            let mut constraints = vec![(left_ty, output_ty)];

            // constrain each argument
            for (arg_ident, arg) in call.args.iter().zip(signature.inputs.iter()) {
                let constrain_ty = &arg.ty;
                match arg_ident {
                    WCallArg::Ident(ident) => {
                        let arg_ty = get_type(types, ident)?;
                        constraints.push((arg_ty, constrain_ty.clone()));
                    }
                    WCallArg::Literal(_lit) => {
                        // TODO: constrain literal call argument
                    }
                }
            }

            for (type_a, type_b) in constraints {
                self.add_eq_constraint(type_a, type_b);
            }

            Ok(())
        } else {
            // not found
            Err(Error::new(
                ErrorType::UnknownCallFunction(format!("{:?}", call_path)),
                call_path.span(),
            ))
        }
    }
}

fn get_type(types: &BTreeMap<WIdent, WTypeId>, ident: &WIdent) -> Result<WTypeId, Error> {
    if let Some(global_ty) = types.get(ident) {
        return Ok(global_ty.clone());
    }

    Err(Error::new(
        ErrorType::UndefinedVariable(ident.name().to_string()),
        ident.span(),
    ))
}
