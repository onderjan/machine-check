use anyhow::anyhow;
use btor2rs::{BiOp, BiOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::util::{create_rnid_expr, create_value_expr};

use super::NodeTranslator;

impl<'a> NodeTranslator<'a> {
    pub fn bi_op_expr(&self, op: &BiOp) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rnid_expr(op.a);
        let b_tokens = create_rnid_expr(op.b);
        match op.ty {
            BiOpType::Iff => {
                Ok(parse_quote!(::mck::forward::TypedEq::typed_eq(#a_tokens, #b_tokens)))
            }
            BiOpType::Implies => {
                // a implies b = !a | b
                let not_a: Expr = parse_quote!(::mck::forward::Bitwise::bit_not(#a_tokens));
                Ok(parse_quote!(::mck::forward::Bitwise::bit_or(#not_a, #b_tokens)))
            }
            BiOpType::Eq => {
                Ok(parse_quote!(::mck::forward::TypedEq::typed_eq(#a_tokens, #b_tokens)))
            }
            BiOpType::Neq => Ok(
                parse_quote!(::mck::forward::Bitwise::bit_not(::mck::forward::TypedEq::typed_eq(#a_tokens, #b_tokens))),
            ),
            // implement greater using lesser by flipping the operands
            BiOpType::Sgt => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_slt(#b_tokens, #a_tokens)))
            }
            BiOpType::Ugt => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_ult(#b_tokens, #a_tokens)))
            }
            BiOpType::Sgte => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_slte(#b_tokens, #a_tokens)))
            }
            BiOpType::Ugte => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_ulte(#b_tokens, #a_tokens)))
            }
            // lesser is implemented
            BiOpType::Slt => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_slt(#a_tokens, #b_tokens)))
            }
            BiOpType::Ult => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_ult(#a_tokens, #b_tokens)))
            }
            BiOpType::Slte => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_slte(#a_tokens, #b_tokens)))
            }
            BiOpType::Ulte => {
                Ok(parse_quote!(::mck::forward::TypedCmp::typed_ulte(#a_tokens, #b_tokens)))
            }
            BiOpType::And => {
                Ok(parse_quote!(::mck::forward::Bitwise::bit_and(#a_tokens, #b_tokens)))
            }
            BiOpType::Nand => {
                let pos: Expr =
                    parse_quote!(::mck::forward::Bitwise::bit_and(#a_tokens, #b_tokens));
                Ok(parse_quote!(::mck::forward::Bitwise::bit_not(#pos)))
            }
            BiOpType::Or => Ok(parse_quote!(::mck::forward::Bitwise::bit_or(#a_tokens, #b_tokens))),
            BiOpType::Nor => {
                let pos: Expr = parse_quote!(::mck::forward::Bitwise::bit_or(#a_tokens, #b_tokens));
                Ok(parse_quote!(::mck::forward::Bitwise::bit_not(#pos)))
            }
            BiOpType::Xor => {
                Ok(parse_quote!(::mck::forward::Bitwise::bit_xor(#a_tokens, #b_tokens)))
            }
            BiOpType::Xnor => {
                let pos: Expr =
                    parse_quote!(::mck::forward::Bitwise::bit_xor(#a_tokens, #b_tokens));
                Ok(parse_quote!(::mck::forward::Bitwise::bit_not(#pos)))
            }
            BiOpType::Rol => Err(anyhow!("Left rotation generation not implemented")),
            BiOpType::Ror => Err(anyhow!("Right rotation generation not implemented")),
            BiOpType::Sll => {
                Ok(parse_quote!(::mck::forward::HwShift::logic_shl(#a_tokens, #b_tokens)))
            }
            BiOpType::Sra => {
                Ok(parse_quote!(::mck::forward::HwShift::arith_shr(#a_tokens, #b_tokens)))
            }
            BiOpType::Srl => {
                Ok(parse_quote!(::mck::forward::HwShift::logic_shr(#a_tokens, #b_tokens)))
            }
            BiOpType::Add => Ok(parse_quote!(::mck::forward::HwArith::add(#a_tokens, #b_tokens))),
            BiOpType::Sub => Ok(parse_quote!(::mck::forward::HwArith::sub(#a_tokens, #b_tokens))),
            BiOpType::Mul => Ok(parse_quote!(::mck::forward::HwArith::mul(#a_tokens, #b_tokens))),
            BiOpType::Sdiv => Ok(parse_quote!(::mck::forward::HwArith::sdiv(#a_tokens, #b_tokens))),
            BiOpType::Udiv => Ok(parse_quote!(::mck::forward::HwArith::udiv(#a_tokens, #b_tokens))),
            BiOpType::Smod => Err(anyhow!("Smod operation generation not implemented")),
            BiOpType::Srem => Ok(parse_quote!(::mck::forward::HwArith::srem(#a_tokens, #b_tokens))),
            BiOpType::Urem => Ok(parse_quote!(::mck::forward::HwArith::urem(#a_tokens, #b_tokens))),
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
                let result_sort = self.get_bitvec(op.sid)?;
                let result_length = result_sort.length.get();

                // do unsigned extension of both to result type
                let a_uext: Expr =
                    parse_quote!(::mck::forward::Ext::<#result_length>::uext(#a_tokens));
                let b_uext: Expr =
                    parse_quote!(::mck::forward::Ext::<#result_length>::uext(#b_tokens));

                // shift a left by length of b
                let b_sort = self.get_nid_bitvec(op.b.nid())?;
                let b_length = b_sort.length.get();
                let shift_length_expr = create_value_expr(b_length.into(), result_sort);
                let a_uext_sll: Expr =
                    parse_quote!(::mck::forward::HwShift::logic_shl(#a_uext, #shift_length_expr));

                // bit-or together
                Ok(parse_quote!(::mck::forward::Bitwise::bit_or(#a_uext_sll, #b_uext)))
            }
            BiOpType::Read => Err(anyhow!("Generating arrays not supported")),
        }
    }
}
