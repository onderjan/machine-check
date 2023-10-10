use anyhow::anyhow;
use btor2rs::{
    BiOp, BiOpType, Bitvec, Const, DrainType, ExtOp, Nid, Node, Sid, SliceOp, Sort, SourceType,
    TriOp, TriOpType, UniOp, UniOpType,
};
use syn::{parse_quote, Expr};

use super::{
    util::{create_nid_ident, create_rnid_expr, create_value_expr},
    Transcriber,
};

pub(super) fn transcribe(
    transcriber: &Transcriber,
    for_init: bool,
) -> Result<Vec<syn::Stmt>, anyhow::Error> {
    let mut transcription = StmtTranscriber {
        transcriber,
        stmts: Vec::new(),
        for_init,
    };
    for (nid, node) in transcriber.btor2.nodes.iter() {
        transcription.transcribe_node(*nid, node)?;
    }
    Ok(transcription.stmts)
}

struct StmtTranscriber<'a> {
    transcriber: &'a Transcriber,
    stmts: Vec<syn::Stmt>,
    for_init: bool,
}

impl<'a> StmtTranscriber<'a> {
    pub fn transcribe_node(&mut self, nid: Nid, node: &Node) -> Result<(), anyhow::Error> {
        let result_ident = create_nid_ident(nid);
        match node {
            Node::State(_) => {
                // the state info should definitely be present for the state
                let state_info = self.transcriber.state_info_map.get(&nid).unwrap();

                let treat_as_input = if self.for_init {
                    if let Some(init) = state_info.init {
                        let init_expr = create_rnid_expr(init);
                        self.stmts
                            .push(parse_quote!(let #result_ident = #init_expr;));
                        false
                    } else {
                        true
                    }
                } else if state_info.next.is_some() {
                    let state_ident = create_nid_ident(nid);
                    self.stmts
                        .push(parse_quote!(let #result_ident = state.#state_ident;));
                    false
                } else {
                    true
                };
                if treat_as_input {
                    let input_ident = create_nid_ident(nid);
                    self.stmts
                        .push(parse_quote!(let #result_ident = input.#input_ident;));
                }
            }
            Node::Const(const_value) => {
                let const_tokens = self.const_expr(const_value)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #const_tokens;));
            }
            Node::Drain(drain) => {
                match drain.ty {
                    DrainType::Output | DrainType::Bad | DrainType::Constraint => {
                        // outputs are unimportant
                        // bad and constraint are treated on their own
                    }
                    DrainType::Fair => {
                        return Err(anyhow!("Fairness constraints not supported"));
                    }
                }
            }
            Node::ExtOp(op) => {
                let expression = self.ext_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::SliceOp(op) => {
                let expression = self.slice_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::UniOp(op) => {
                let expression = self.uni_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::BiOp(op) => {
                let expression = self.bi_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::TriOp(op) => {
                let expression = self.tri_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::Source(source) => match source.ty {
                SourceType::Input => {
                    // move from input
                    let input_ident = create_nid_ident(nid);
                    self.stmts
                        .push(parse_quote!(let #result_ident = input.#input_ident;));
                }
                SourceType::One => {
                    let expr = create_value_expr(1, self.get_nid_bitvec(nid)?);
                    self.stmts.push(parse_quote!(let #result_ident = #expr;));
                }
                SourceType::Ones => {
                    // negate one
                    let expr = create_value_expr(1, self.get_nid_bitvec(nid)?);
                    self.stmts.push(parse_quote!(let #result_ident = -#expr;));
                }
                SourceType::Zero => {
                    let expr = create_value_expr(0, self.get_nid_bitvec(nid)?);
                    self.stmts.push(parse_quote!(let #result_ident = #expr;));
                }
            },
            Node::Temporal(_) => {
                // not handled here
            }
            Node::Justice(_) => return Err(anyhow!("Justice not implemented")),
        }
        Ok(())
    }

    pub fn uni_op_expr(&self, op: &UniOp) -> Result<syn::Expr, anyhow::Error> {
        let result_bitvec = self.get_bitvec(op.sid)?;
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;

        let a_tokens = create_rnid_expr(op.a);
        match op.ty {
            UniOpType::Not => Ok(parse_quote!(!(#a_tokens))),
            UniOpType::Inc => {
                let one = create_value_expr(1, result_bitvec);
                Ok(parse_quote!((#a_tokens) + (#one)))
            }
            UniOpType::Dec => {
                let one = create_value_expr(1, result_bitvec);
                Ok(parse_quote!((#a_tokens) - (#one)))
            }
            UniOpType::Neg => Ok(parse_quote!(-(#a_tokens))),
            UniOpType::Redand => {
                // equality with all ones (equivalent to wrapping minus one)
                // sort for constant is taken from the operand, not result
                let one = create_value_expr(1, a_bitvec);
                Ok(parse_quote!(::mck::TypedEq::typed_eq(#a_tokens, -#one)))
            }
            UniOpType::Redor => {
                // inequality with all zeros
                // sort for constant is taken from the operand, not result
                let zero = create_value_expr(0, a_bitvec);
                Ok(parse_quote!(!(::mck::TypedEq::typed_eq(#a_tokens, #zero))))
            }
            UniOpType::Redxor => {
                // naive version, just slice all relevant bits and xor them together
                let a_length = a_bitvec.length.get();
                let mut slice_expressions = Vec::<syn::Expr>::new();
                let a_tokens = create_rnid_expr(op.a);
                for i in 0..a_length {
                    // logical shift right to make the i the zeroth bit
                    let shift_length_expr = create_value_expr(i.into(), a_bitvec);
                    let a_srl: Expr =
                        parse_quote!(::mck::MachineShift::srl(#a_tokens, #shift_length_expr));
                    slice_expressions.push(parse_quote!(::mck::MachineExt::<1>::uext(#a_srl)));
                }
                Ok(parse_quote!(#(#slice_expressions)^*))
            }
        }
    }

    pub fn tri_op_expr(&self, op: &TriOp) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rnid_expr(op.a);
        let b_tokens = create_rnid_expr(op.b);
        let c_tokens = create_rnid_expr(op.c);
        match op.ty {
            TriOpType::Ite => {
                // a = condition, b = then, c = else
                // to avoid control flow, convert condition to bitmask

                let result_sort = self.get_bitvec(op.sid)?;
                let result_length = result_sort.length.get();
                let condition_mask: Expr =
                    parse_quote!(::mck::MachineExt::<#result_length>::sext(#a_tokens));
                let neg_condition_mask: Expr =
                    parse_quote!(::mck::MachineExt::<#result_length>::sext(!(#a_tokens)));

                Ok(
                    parse_quote!(((#b_tokens) & (#condition_mask)) | ((#c_tokens) & (#neg_condition_mask))),
                )
            }
            TriOpType::Write => {
                // a = array, b = index, c = element to be stored
                Err(anyhow!("Generating arrays not supported"))
            }
        }
    }

    pub fn bi_op_expr(&self, op: &BiOp) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rnid_expr(op.a);
        let b_tokens = create_rnid_expr(op.b);
        match op.ty {
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
                let result_sort: &Bitvec = self.get_bitvec(op.sid)?;
                let result_length = result_sort.length.get();

                // do unsigned extension of both to result type
                let a_uext: Expr =
                    parse_quote!(::mck::MachineExt::<#result_length>::uext(#a_tokens));
                let b_uext: Expr =
                    parse_quote!(::mck::MachineExt::<#result_length>::uext(#b_tokens));

                // shift a left by length of b
                let b_sort: &Bitvec = self.get_nid_bitvec(op.b.nid())?;
                let b_length = b_sort.length.get();
                let shift_length_expr = create_value_expr(b_length.into(), result_sort);
                let a_uext_sll: Expr =
                    parse_quote!(::mck::MachineShift::sll(#a_uext, #shift_length_expr));

                // bit-or together
                Ok(parse_quote!((#a_uext_sll) | (#b_uext)))
            }
            BiOpType::Read => Err(anyhow!("Generating arrays not supported")),
        }
    }

    pub fn ext_op_expr(&self, op: &ExtOp) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rnid_expr(op.a);

        // just compute the new number of bits and perform the extension
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;
        let a_length = a_bitvec.length.get();
        let result_length = a_length + op.length;

        match op.ty {
            btor2rs::ExtOpType::Sext => {
                Ok(parse_quote!(::mck::MachineExt::<#result_length>::sext(#a_tokens)))
            }
            btor2rs::ExtOpType::Uext => {
                Ok(parse_quote!(::mck::MachineExt::<#result_length>::uext(#a_tokens)))
            }
        }
    }

    pub fn slice_op_expr(&self, op: &SliceOp) -> Result<syn::Expr, anyhow::Error> {
        let a_sort = self.get_nid_bitvec(op.a.nid())?;
        let a_tokens = create_rnid_expr(op.a);

        // logical shift right to make the lower bit the zeroth bit
        let shift_length_expr = create_value_expr(op.lower_bit.into(), a_sort);
        let a_srl: Expr = parse_quote!(::mck::MachineShift::srl(#a_tokens, #shift_length_expr));

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = op.upper_bit - op.lower_bit + 1;

        Ok(parse_quote!(::mck::MachineExt::<#num_retained_bits>::uext(#a_srl)))
    }

    fn const_expr(&self, value: &Const) -> Result<Expr, anyhow::Error> {
        let result_sort = self.get_bitvec(value.sid)?;
        // parse the value first to disallow hijinks
        // convert negation to negation of resulting bitvector
        let (negate, str) = if let Some(str) = value.string.strip_prefix('-') {
            (true, str)
        } else {
            (false, value.string.as_str())
        };

        let value = u64::from_str_radix(str, value.ty.clone() as u32)?;
        let bitvec_length = result_sort.length.get();
        Ok(if negate {
            parse_quote!((-::mck::MachineBitvector::<#bitvec_length>::new(#value)))
        } else {
            parse_quote!(::mck::MachineBitvector::<#bitvec_length>::new(#value))
        })
    }

    fn get_sort(&self, sid: Sid) -> Result<&Sort, anyhow::Error> {
        self.transcriber
            .btor2
            .sorts
            .get(&sid)
            .ok_or_else(|| anyhow!("Unknown sort"))
    }

    fn get_bitvec(&self, sid: Sid) -> Result<&Bitvec, anyhow::Error> {
        let sort = self.get_sort(sid)?;
        let Sort::Bitvec(bitvec) = sort else {
            return Err(anyhow!("Expected bitvec sort"));
        };
        Ok(bitvec)
    }

    fn get_nid_bitvec(&self, nid: Nid) -> Result<&Bitvec, anyhow::Error> {
        let node = self
            .transcriber
            .btor2
            .nodes
            .get(&nid)
            .ok_or_else(|| anyhow!("Unknown node"))?;
        let sid = node
            .get_sid()
            .ok_or_else(|| anyhow!("Expected node with sid"))?;
        self.get_bitvec(sid)
    }
}
