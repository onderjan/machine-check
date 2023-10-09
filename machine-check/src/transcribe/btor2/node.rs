use anyhow::anyhow;
use btor2rs::{
    id::Nid,
    node::{Const, Node, NodeType},
    refs::Lref,
    sort::Sort,
    BiOp, BiOpType, Btor2, ExtOp, SliceOp, TriOp, TriOpType, UniOp, UniOpType,
};
use syn::{parse_quote, Expr};

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
        let result_ident = nid.create_ident("node");
        match &node.ntype {
            NodeType::State(state) => {
                let treat_as_input = if self.for_init {
                    if let Some(init) = state.init() {
                        let init_tokens = init.create_tokens("node");
                        self.stmts
                            .push(parse_quote!(let #result_ident = #init_tokens;));
                        false
                    } else {
                        true
                    }
                } else if state.next().is_some() {
                    let state_ident = nid.create_ident("state");
                    self.stmts
                        .push(parse_quote!(let #result_ident = state.#state_ident;));
                    false
                } else {
                    true
                };
                if treat_as_input {
                    let input_ident = nid.create_ident("input");
                    self.stmts
                        .push(parse_quote!(let #result_ident = input.#input_ident;));
                }
            }
            NodeType::Const(const_value) => {
                let Sort::Bitvec(bitvec) = &node.result.sort else {
                // just here to be sure, should not happen
                return Err(anyhow::anyhow!("Expected bitvec const value, but have {:?}", node.result.sort));
            };
                let const_tokens = const_value.create_tokens(bitvec);
                self.stmts
                    .push(parse_quote!(let #result_ident = #const_tokens;));
            }
            NodeType::Input => {
                let input_ident = nid.create_ident("input");
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
        let a_tokens = op.a.create_tokens("node");
        let Sort::Bitvec(result_bitvec) = result_sort else {
            return Err(anyhow!("Expected bitvec result, but have {:?}", result_sort));
        };
        let Sort::Bitvec(a_bitvec) = &op.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {:?}", op.a.sort));
        };
        match op.op_type {
            UniOpType::Not => Ok(parse_quote!(!(#a_tokens))),
            UniOpType::Inc => {
                let one = Const::new(false, 1).create_tokens(result_bitvec);
                Ok(parse_quote!((#a_tokens) + (#one)))
            }
            UniOpType::Dec => {
                let one = Const::new(false, 1).create_tokens(result_bitvec);
                Ok(parse_quote!((#a_tokens) - (#one)))
            }
            UniOpType::Neg => Ok(parse_quote!(-(#a_tokens))),
            UniOpType::Redand => {
                // equality with all ones
                // sort for constant is taken from the operand, not result
                let all_ones_const = Const::new(true, 1);
                let all_ones_tokens = all_ones_const.create_tokens(a_bitvec);

                Ok(parse_quote!(::mck::TypedEq::typed_eq(#a_tokens, #all_ones_tokens)))
            }
            UniOpType::Redor => {
                // inequality with all zeros
                // sort for constant is taken from the operand, not result
                let all_zeros_const = Const::new(false, 0);
                let all_zeros_tokens = all_zeros_const.create_tokens(a_bitvec);

                Ok(parse_quote!(!(::mck::TypedEq::typed_eq(#a_tokens, #all_zeros_tokens))))
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
        let result_ident = result.create_ident("node");
        let a_tokens = op.a.create_tokens("node");
        let b_tokens = op.b.create_tokens("node");
        let c_tokens = op.c.create_tokens("node");
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
        let a_tokens = op.a.create_tokens("node");
        let b_tokens = op.b.create_tokens("node");
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
                let Sort::Bitvec(result_sort) = result_sort else {
                return Err(anyhow!("Expected bitvec result, but have {:?}", result_sort));
            };
                let result_length = result_sort.length.get();

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

                let sll_const = Const::new(false, b_length as u64);
                let sll_tokens = sll_const.create_tokens(result_sort);
                let a_uext_sll: Expr = parse_quote!(::mck::MachineShift::sll(#a_uext, #sll_tokens));

                // bit-or together
                Ok(parse_quote!((#a_uext_sll) | (#b_uext)))
            }
            BiOpType::Read => Err(anyhow!("Generating arrays not supported")),
        }
    }

    pub fn ext_op_expr(&self, op: &ExtOp, result_sort: &Sort) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = op.a.create_tokens("node");

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
        let a_tokens = op.a.create_tokens("node");
        let Sort::Bitvec(a_bitvec) = &op.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {:?}", result_sort));
        };

        // logical shift right to make the lower bit the zeroth bit
        let srl_const = Const::new(false, op.lower_bit as u64);
        let srl_tokens = srl_const.create_tokens(a_bitvec);
        let a_srl: Expr = parse_quote!(::mck::MachineShift::srl(#a_tokens, #srl_tokens));

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = op.upper_bit - op.lower_bit + 1;

        Ok(parse_quote!(::mck::MachineExt::<#num_retained_bits>::uext(#a_srl)))
    }
}
