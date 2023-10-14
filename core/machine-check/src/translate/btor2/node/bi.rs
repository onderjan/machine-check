use anyhow::anyhow;
use btor2rs::{BiOp, BiOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::util::create_rnid_expr;

use super::{constant::create_value_expr, ext::create_uext, uni::create_bit_not, NodeTranslator};

impl<'a> NodeTranslator<'a> {
    pub fn bi_op_expr(&self, op: &BiOp) -> Result<syn::Expr, anyhow::Error> {
        let a_expr = create_rnid_expr(op.a);
        let b_expr = create_rnid_expr(op.b);
        Ok(match op.ty {
            BiOpType::Iff => create_eq(a_expr, b_expr),
            BiOpType::Implies => {
                // a implies b = !a | b
                let not_a = create_bit_not(a_expr);
                create_eq(not_a, b_expr)
            }
            BiOpType::Eq => create_eq(a_expr, b_expr),
            BiOpType::Neq => create_bit_not(create_eq(a_expr, b_expr)),
            // lesser is implemented
            BiOpType::Ult => create_ult(a_expr, b_expr),
            BiOpType::Ulte => create_ulte(a_expr, b_expr),
            BiOpType::Slt => create_slt(a_expr, b_expr),
            BiOpType::Slte => create_slte(a_expr, b_expr),
            // implement greater using lesser by flipping the operands
            BiOpType::Ugt => create_ult(b_expr, a_expr),
            BiOpType::Ugte => create_ulte(b_expr, a_expr),
            BiOpType::Sgt => create_slt(b_expr, a_expr),
            BiOpType::Sgte => create_slte(b_expr, a_expr),
            BiOpType::And => create_bit_and(a_expr, b_expr),
            BiOpType::Nand => create_bit_not(create_bit_and(a_expr, b_expr)),
            BiOpType::Or => create_bit_or(a_expr, b_expr),
            BiOpType::Nor => create_bit_not(create_bit_or(a_expr, b_expr)),
            BiOpType::Xor => create_bit_xor(a_expr, b_expr),
            BiOpType::Xnor => create_bit_not(create_bit_xor(a_expr, b_expr)),
            BiOpType::Rol => return Err(anyhow!("Left rotation generation not implemented")),
            BiOpType::Ror => return Err(anyhow!("Right rotation generation not implemented")),
            BiOpType::Sll => create_logic_shl(a_expr, b_expr),
            BiOpType::Srl => create_logic_shr(a_expr, b_expr),
            BiOpType::Sra => create_arith_shr(a_expr, b_expr),
            BiOpType::Add => create_add(a_expr, b_expr),
            BiOpType::Sub => create_sub(a_expr, b_expr),
            BiOpType::Mul => create_mul(a_expr, b_expr),
            BiOpType::Udiv => create_udiv(a_expr, b_expr),
            BiOpType::Urem => create_urem(a_expr, b_expr),
            BiOpType::Srem => create_srem(a_expr, b_expr),
            BiOpType::Sdiv => create_sdiv(a_expr, b_expr),
            BiOpType::Smod => return Err(anyhow!("Smod operation generation not implemented")),
            BiOpType::Saddo
            | BiOpType::Uaddo
            | BiOpType::Sdivo
            | BiOpType::Udivo
            | BiOpType::Smulo
            | BiOpType::Umulo
            | BiOpType::Ssubo
            | BiOpType::Usubo => {
                return Err(anyhow!("Overflow operation generation not implemented"))
            }
            BiOpType::Concat => self.concat_expr(op, a_expr, b_expr)?,
            BiOpType::Read => return Err(anyhow!("Generating arrays not supported")),
        })
    }

    fn concat_expr(&self, op: &BiOp, a_expr: Expr, b_expr: Expr) -> Result<Expr, anyhow::Error> {
        // a is the higher, b is the lower
        let result_sort = self.get_bitvec(op.sid)?;
        let result_length = result_sort.length.get();

        // do unsigned extension of both to result type
        let a_uext = create_uext(a_expr, result_length);
        let b_uext: Expr = create_uext(b_expr, result_length);

        // shift a left by length of b
        let b_sort = self.get_nid_bitvec(op.b.nid())?;
        let b_length = b_sort.length.get();
        let shift_length_expr = create_value_expr(b_length.into(), result_sort);
        let a_uext_sll: Expr = create_logic_shl(a_uext, shift_length_expr);
        // bit-or together
        Ok(create_bit_or(a_uext_sll, b_uext))
    }
}

// equality
pub(super) fn create_eq(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::TypedEq::typed_eq(#a_expr, #b_expr))
}

// comparison
pub(super) fn create_ult(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::TypedEq::typed_ult(#a_expr, #b_expr))
}

pub(super) fn create_ulte(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::TypedEq::typed_ulte(#a_expr, #b_expr))
}

pub(super) fn create_slt(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::TypedEq::typed_slt(#a_expr, #b_expr))
}

pub(super) fn create_slte(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::TypedEq::typed_slte(#a_expr, #b_expr))
}

// bitwise
pub(super) fn create_bit_and(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::Bitwise::bit_and(#a_expr, #b_expr))
}

pub(super) fn create_bit_or(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::Bitwise::bit_or(#a_expr, #b_expr))
}

pub(super) fn create_bit_xor(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::Bitwise::bit_xor(#a_expr, #b_expr))
}

// arith
pub(super) fn create_add(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::add(#a_expr, #b_expr))
}

pub(super) fn create_sub(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::sub(#a_expr, #b_expr))
}

pub(super) fn create_mul(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::mul(#a_expr, #b_expr))
}

pub(super) fn create_udiv(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::udiv(#a_expr, #b_expr))
}

pub(super) fn create_urem(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::urem(#a_expr, #b_expr))
}

pub(super) fn create_sdiv(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::sdiv(#a_expr, #b_expr))
}

pub(super) fn create_srem(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::srem(#a_expr, #b_expr))
}

// shift

pub(super) fn create_logic_shl(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwShift::logic_shl(#a_expr, #b_expr))
}
pub(super) fn create_logic_shr(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwShift::logic_shr(#a_expr, #b_expr))
}
pub(super) fn create_arith_shr(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!(::mck::forward::HwShift::arith_shr(#a_expr, #b_expr))
}
