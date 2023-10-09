use anyhow::anyhow;
use btor2rs::{
    BiOp, BiOpType, Btor2, Const, ExtOp, Lref, Nid, Node, NodeType, Rref, SliceOp, Sort, TriOp,
    TriOpType, UniOp, UniOpType,
};
use proc_macro2::Span;
use syn::{parse_quote, Expr, Ident, Type};

pub fn transcribe(btor2: &Btor2, for_init: bool) -> Result<Vec<syn::Stmt>, anyhow::Error> {
    let mut transcription = Transcription {
        stmts: Vec::new(),
        for_init,
    };
    for (nid, node) in btor2.nodes.iter() {
        transcription.transcribe_node(nid, node)?;
    }
    Ok(transcription.stmts)
}

struct Transcription {
    stmts: Vec<syn::Stmt>,
    for_init: bool,
}

impl Transcription {
    pub fn transcribe_node(&mut self, nid: &Nid, node: &Node) -> Result<(), anyhow::Error> {
        let result_ident = create_nid_ident(nid, "node");
        match &node.ntype {
            NodeType::State(state) => {
                let treat_as_input = if self.for_init {
                    if let Some(init) = state.init() {
                        let init_tokens = create_rref_expr(init, "node");
                        self.stmts
                            .push(parse_quote!(let #result_ident = #init_tokens;));
                        false
                    } else {
                        true
                    }
                } else if state.next().is_some() {
                    let state_ident = create_nid_ident(nid, "state");
                    self.stmts
                        .push(parse_quote!(let #result_ident = state.#state_ident;));
                    false
                } else {
                    true
                };
                if treat_as_input {
                    let input_ident = create_nid_ident(nid, "input");
                    self.stmts
                        .push(parse_quote!(let #result_ident = input.#input_ident;));
                }
            }
            NodeType::Const(const_value) => {
                let const_tokens = create_const_expr(const_value, &node.result.sort)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #const_tokens;));
            }
            NodeType::Input => {
                let input_ident = create_nid_ident(nid, "input");
                self.stmts
                    .push(parse_quote!(let #result_ident = input.#input_ident;));
            }
            NodeType::Output(_) => {
                // outputs are unimportant for verification
            }
            NodeType::ExtOp(op) => {
                let expression = self.ext_op_expr(op, &node.result.sort)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            NodeType::SliceOp(op) => {
                let expression = self.slice_op_expr(op, &node.result.sort)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            NodeType::UniOp(op) => {
                let expression = self.uni_op_expr(op, &node.result.sort)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            NodeType::BiOp(op) => {
                let expression = self.bi_op_expr(op, &node.result.sort)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            NodeType::TriOp(op) => {
                let statement = self.tri_op_expr(op, &node.result)?;
                self.stmts.push(statement);
            }
            NodeType::Bad(_) => {
                // bad is treated in its own function
            }
            NodeType::Constraint(_) => {
                // constraints are treated at the end
            }
        }
        Ok(())
    }

    pub fn uni_op_expr(&self, op: &UniOp, result_sort: &Sort) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rref_expr(&op.a, "node");
        let Sort::Bitvec(result_bitvec) = result_sort else {
            return Err(anyhow!("Expected bitvec result, but have {:?}", result_sort));
        };
        let Sort::Bitvec(_) = &op.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {:?}", op.a.sort));
        };
        match op.op_type {
            UniOpType::Not => Ok(parse_quote!(!(#a_tokens))),
            UniOpType::Inc => {
                let one = create_const_one(result_sort)?;
                Ok(parse_quote!((#a_tokens) + (#one)))
            }
            UniOpType::Dec => {
                let one = create_const_one(result_sort)?;
                Ok(parse_quote!((#a_tokens) - (#one)))
            }
            UniOpType::Neg => Ok(parse_quote!(-(#a_tokens))),
            UniOpType::Redand => {
                // equality with all ones
                // sort for constant is taken from the operand, not result
                let all_ones = create_const_all_ones(result_sort)?;

                Ok(parse_quote!(::mck::TypedEq::typed_eq(#a_tokens, #all_ones)))
            }
            UniOpType::Redor => {
                // inequality with all zeros
                // sort for constant is taken from the operand, not result
                let zero = create_const_zero(result_sort)?;
                Ok(parse_quote!(!(::mck::TypedEq::typed_eq(#a_tokens, #zero))))
            }
            UniOpType::Redxor => {
                // naive version, just slice all relevant bits and xor them together
                let bitvec_length = result_bitvec.length.get();
                let mut slice_expressions = Vec::<syn::Expr>::new();
                let single_bit_sort = Sort::single_bit_sort();
                for i in 0..bitvec_length {
                    let i_slice = SliceOp {
                        a: op.a.clone(),
                        lower_bit: i,
                        upper_bit: i,
                    };
                    let i_unparenthesised_expression =
                        self.slice_op_expr(&i_slice, &single_bit_sort)?;
                    slice_expressions.push(parse_quote!((#i_unparenthesised_expression)));
                }
                Ok(parse_quote!(#(#slice_expressions)^*))
            }
        }
    }

    pub fn tri_op_expr(&self, op: &TriOp, result: &Lref) -> Result<syn::Stmt, anyhow::Error> {
        let result_ident = create_lref_ident(result, "node");
        let a_tokens = create_rref_expr(&op.a, "node");
        let b_tokens = create_rref_expr(&op.b, "node");
        let c_tokens = create_rref_expr(&op.c, "node");
        match op.op_type {
            TriOpType::Ite => {
                // a = condition, b = then, c = else
                // to avoid control flow, convert condition to bitmask
                let Sort::Bitvec(bitvec) = &result.sort else {
                    return Err(anyhow!("Expected bitvec result, but have {:?}", result.sort));
                };
                let bitvec_length = bitvec.length.get();
                let condition_mask: Expr =
                    parse_quote!(::mck::MachineExt::<#bitvec_length>::sext(#a_tokens));
                let neg_condition_mask: Expr =
                    parse_quote!(::mck::MachineExt::<#bitvec_length>::sext(!(#a_tokens)));

                Ok(
                    parse_quote!(let #result_ident = ((#b_tokens) & (#condition_mask)) | ((#c_tokens) & (#neg_condition_mask));),
                )
            }
            TriOpType::Write => {
                // a = array, b = index, c = element to be stored
                Err(anyhow!("Generating arrays not supported"))
            }
        }
    }

    pub fn bi_op_expr(&self, op: &BiOp, result_sort: &Sort) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rref_expr(&op.a, "node");
        let b_tokens = create_rref_expr(&op.b, "node");
        match op.op_type {
            BiOpType::Iff => Ok(parse_quote!(::mck::TypedEq::typed_eq(#a_tokens, #b_tokens))),
            BiOpType::Implies => Ok(parse_quote!(!(#a_tokens) | (#b_tokens))),
            BiOpType::Eq => Ok(parse_quote!(::mck::TypedEq::typed_eq(#a_tokens, #b_tokens))),
            BiOpType::Neq => Ok(parse_quote!(!(::mck::TypedEq::typed_eq(#a_tokens, #b_tokens)))),
            // implement greater using lesser by flipping the operands
            BiOpType::Sgt => Ok(parse_quote!(::mck::TypedCmp::typed_slt(#b_tokens, #a_tokens))),
            BiOpType::Ugt => Ok(parse_quote!(::mck::TypedCmp::typed_ult(#b_tokens, #a_tokens))),
            BiOpType::Sgte => Ok(parse_quote!(::mck::TypedCmp::typed_slte(#b_tokens, #a_tokens))),
            BiOpType::Ugte => Ok(parse_quote!(::mck::TypedCmp::typed_ulte(#b_tokens, #a_tokens))),
            // lesser is implemented
            BiOpType::Slt => Ok(parse_quote!(::mck::TypedCmp::typed_slt(#a_tokens, #b_tokens))),
            BiOpType::Ult => Ok(parse_quote!(::mck::TypedCmp::typed_ult(#a_tokens, #b_tokens))),
            BiOpType::Slte => Ok(parse_quote!(::mck::TypedCmp::typed_slte(#a_tokens, #b_tokens))),
            BiOpType::Ulte => Ok(parse_quote!(::mck::TypedCmp::typed_ulte(#a_tokens, #b_tokens))),
            BiOpType::And => Ok(parse_quote!((#a_tokens) & (#b_tokens))),
            BiOpType::Nand => Ok(parse_quote!(!((#a_tokens) & (#b_tokens)))),
            BiOpType::Nor => Ok(parse_quote!(!((#a_tokens) | (#b_tokens)))),
            BiOpType::Or => Ok(parse_quote!((#a_tokens) | (#b_tokens))),
            BiOpType::Xnor => Ok(parse_quote!(!((#a_tokens) ^ (#b_tokens)))),
            BiOpType::Xor => Ok(parse_quote!((#a_tokens) ^ (#b_tokens))),
            BiOpType::Rol => Err(anyhow!("Left rotation generation not implemented")),
            BiOpType::Ror => Err(anyhow!("Right rotation generation not implemented")),
            BiOpType::Sll => Ok(parse_quote!(::mck::MachineShift::sll(#a_tokens, #b_tokens))),
            BiOpType::Sra => Ok(parse_quote!(::mck::MachineShift::sra(#a_tokens, #b_tokens))),
            BiOpType::Srl => Ok(parse_quote!(::mck::MachineShift::srl(#a_tokens, #b_tokens))),
            BiOpType::Add => Ok(parse_quote!((#a_tokens) + (#b_tokens))),
            BiOpType::Mul => Ok(parse_quote!((#a_tokens) * (#b_tokens))),
            BiOpType::Sdiv => Ok(parse_quote!(::mck::MachineDiv::sdiv(#a_tokens, #b_tokens))),
            BiOpType::Udiv => Ok(parse_quote!(::mck::MachineDiv::udiv(#a_tokens, #b_tokens))),
            BiOpType::Smod => Ok(parse_quote!(::mck::MachineDiv::smod(#a_tokens, #b_tokens))),
            BiOpType::Srem => Ok(parse_quote!(::mck::MachineDiv::srem(#a_tokens, #b_tokens))),
            BiOpType::Urem => Ok(parse_quote!(::mck::MachineDiv::urem(#a_tokens, #b_tokens))),
            BiOpType::Sub => Ok(parse_quote!((#a_tokens) - (#b_tokens))),
            BiOpType::Saddo
            | BiOpType::Uaddo
            | BiOpType::Sdivo
            | BiOpType::Udivo
            | BiOpType::Smulo
            | BiOpType::Umulo
            | BiOpType::Ssubo
            | BiOpType::Usubo => Err(anyhow!("Overflow operation generation not implemented")),
            BiOpType::Concat => {
                // a is the higher, b is the lower
                let Sort::Bitvec(bitvec_sort) = result_sort else {
                return Err(anyhow!("Expected bitvec result, but have {:?}", result_sort));
            };
                let result_length = bitvec_sort.length.get();

                // do unsigned extension of both to result type
                let a_uext: Expr =
                    parse_quote!(::mck::MachineExt::<#result_length>::uext(#a_tokens));
                let b_uext: Expr =
                    parse_quote!(::mck::MachineExt::<#result_length>::uext(#b_tokens));

                // shift a left by length of b
                let Sort::Bitvec(b_sort) = &op.b.sort else {
                return Err(anyhow!("Expected bitvec second parameter, but have {:?}", op.b.sort));
            };
                let b_length = b_sort.length.get();
                let shift_length_expr = create_const_expr(
                    &Const {
                        ty: btor2rs::ConstType::Decimal,
                        string: b_length.to_string(),
                    },
                    result_sort,
                )?;
                let a_uext_sll: Expr =
                    parse_quote!(::mck::MachineShift::sll(#a_uext, #shift_length_expr));

                // bit-or together
                Ok(parse_quote!((#a_uext_sll) | (#b_uext)))
            }
            BiOpType::Read => Err(anyhow!("Generating arrays not supported")),
        }
    }

    pub fn ext_op_expr(&self, op: &ExtOp, result_sort: &Sort) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rref_expr(&op.a, "node");

        // just compute the new number of bits and perform the extension
        let Sort::Bitvec(a_bitvec) = &op.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {:?}", result_sort));
        };
        let a_length = a_bitvec.length.get();

        let result_length = a_length + op.extension_size;

        if op.signed {
            Ok(parse_quote!(::mck::MachineExt::<#result_length>::sext(#a_tokens)))
        } else {
            Ok(parse_quote!(::mck::MachineExt::<#result_length>::uext(#a_tokens)))
        }
    }

    pub fn slice_op_expr(
        &self,
        op: &SliceOp,
        result_sort: &Sort,
    ) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rref_expr(&op.a, "node");
        let Sort::Bitvec(_) = &op.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {:?}", result_sort));
        };

        // logical shift right to make the lower bit the zeroth bit
        let shift_length_expr = create_const_expr(
            &Const {
                ty: btor2rs::ConstType::Decimal,
                string: op.lower_bit.to_string(),
            },
            result_sort,
        )?;
        let a_srl: Expr = parse_quote!(::mck::MachineShift::srl(#a_tokens, #shift_length_expr));

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = op.upper_bit - op.lower_bit + 1;

        Ok(parse_quote!(::mck::MachineExt::<#num_retained_bits>::uext(#a_srl)))
    }
}

pub fn create_nid_ident(nid: &Nid, flavor: &str) -> Ident {
    Ident::new(&format!("{}_{}", flavor, nid.0), Span::call_site())
}

pub fn create_lref_ident(lref: &Lref, flavor: &str) -> Ident {
    create_nid_ident(&lref.nid, flavor)
}

pub fn create_rref_expr(rref: &Rref, flavor: &str) -> Expr {
    let ident = create_nid_ident(&rref.nid, flavor);
    if rref.not {
        parse_quote!((!#ident))
    } else {
        parse_quote!(#ident)
    }
}

pub fn create_const_expr(value: &Const, sort: &Sort) -> Result<Expr, anyhow::Error> {
    // parse the value first to disallow hijinks
    // convert negation to negation of resulting bitvector
    let (negate, str) = if let Some(str) = value.string.strip_prefix('-') {
        (true, str)
    } else {
        (false, value.string.as_str())
    };

    let value = u64::from_str_radix(str, value.ty.clone() as u32)?;
    let Sort::Bitvec(sort) = sort else {
        return Err(anyhow!("Cannot generate constant with array sort"));
    };
    let bitvec_length = sort.length.get();
    Ok(if negate {
        parse_quote!((-::mck::MachineBitvector::<#bitvec_length>::new(#value)))
    } else {
        parse_quote!(::mck::MachineBitvector::<#bitvec_length>::new(#value))
    })
}

pub fn create_const_zero(sort: &Sort) -> Result<Expr, anyhow::Error> {
    create_const_expr(&Const::zero(), sort)
}

pub fn create_const_one(sort: &Sort) -> Result<Expr, anyhow::Error> {
    create_const_expr(&Const::one(), sort)
}

pub fn create_const_all_ones(sort: &Sort) -> Result<Expr, anyhow::Error> {
    create_const_expr(&Const::ones(), sort)
}

pub fn create_sort_type(sort: &Sort) -> Result<Type, anyhow::Error> {
    match sort {
        Sort::Bitvec(bitvec) => {
            let bitvec_length = bitvec.length.get();
            Ok(parse_quote!(::mck::MachineBitvector<#bitvec_length>))
        }
        Sort::Array(_) => Err(anyhow!("Generating arrays not supported")),
    }
}
