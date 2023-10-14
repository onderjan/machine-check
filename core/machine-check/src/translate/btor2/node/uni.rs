use btor2rs::{UniOp, UniOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::util::{create_rnid_expr, create_value_expr, single_bits_xor};

use super::NodeTranslator;

impl<'a> NodeTranslator<'a> {
    pub fn uni_op_expr(&self, op: &UniOp) -> Result<syn::Expr, anyhow::Error> {
        let result_bitvec = self.get_bitvec(op.sid)?;
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;

        let a_tokens = create_rnid_expr(op.a);
        match op.ty {
            UniOpType::Not => Ok(parse_quote!(::mck::forward::Bitwise::bit_not(#a_tokens))),
            UniOpType::Inc => {
                let one = create_value_expr(1, result_bitvec);
                Ok(parse_quote!((::mck::forward::HwArith::add(#a_tokens,#one))))
            }
            UniOpType::Dec => {
                let one = create_value_expr(1, result_bitvec);
                Ok(parse_quote!(::mck::forward::HwArith::sub(#a_tokens, #one)))
            }
            UniOpType::Neg => Ok(parse_quote!(::mck::forward::HwArith::arith_neg(#a_tokens))),
            UniOpType::Redand => {
                // equality with all ones (equivalent to wrapping minus one)
                // sort for constant is taken from the operand, not result
                let one = create_value_expr(1, a_bitvec);
                Ok(
                    parse_quote!(::mck::forward::TypedEq::typed_eq(#a_tokens, ::mck::forward::HwArith::arith_neg(#one))),
                )
            }
            UniOpType::Redor => {
                // inequality with all zeros
                // sort for constant is taken from the operand, not result
                let zero = create_value_expr(0, a_bitvec);
                Ok(
                    parse_quote!(::mck::forward::Bitwise::bit_not(::mck::forward::TypedEq::typed_eq(#a_tokens, #zero))),
                )
            }
            UniOpType::Redxor => {
                // naive version, just slice all relevant bits and XOR them together
                let a_length = a_bitvec.length.get();
                let a_tokens = create_rnid_expr(op.a);

                let slice_exprs = (0..a_length).map(|i| {
                // logical shift right to make the i the zeroth bit
                let shift_length_expr = create_value_expr(i.into(), a_bitvec);
                let a_srl: Expr = parse_quote!(::mck::forward::HwShift::logic_shr(#a_tokens, #shift_length_expr));
                // cut all other bits
                parse_quote!(::mck::forward::Ext::<1>::uext(#a_srl))
            });

                // XOR the bits together
                Ok(single_bits_xor(slice_exprs))
            }
        }
    }
}
