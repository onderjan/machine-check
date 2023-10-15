use btor2rs::{Bitvec, Const};
use syn::{parse_quote, Expr};

use crate::translate::btor2::Error;

use super::{uni::create_arith_neg_expr, NodeTranslator};

impl<'a> NodeTranslator<'a> {
    pub fn const_expr(&self, value: &Const) -> Result<Expr, Error> {
        let result_bitvec = self.get_bitvec(value.sid)?;

        // convert negation to negation of resulting bitvector
        let (negate, str) = if let Some(str) = value.string.strip_prefix('-') {
            (true, str)
        } else {
            (false, value.string.as_str())
        };

        let value = u64::from_str_radix(str, value.ty.clone() as u32)
            .map_err(|_| Error::InvalidConstant(String::from(str)))?;
        // create value and optionally negate it
        let mut value = create_value_expr(value, result_bitvec);
        if negate {
            value = create_arith_neg_expr(value);
        }
        Ok(value)
    }
}

pub fn create_value_expr(value: u64, bitvec: &Bitvec) -> Expr {
    let bitvec_length = bitvec.length.get();
    parse_quote!(::mck::concr::Bitvector::<#bitvec_length>::new(#value))
}

pub fn create_zero_expr(bitvec: &Bitvec) -> Expr {
    create_value_expr(0, bitvec)
}

pub fn create_one_expr(bitvec: &Bitvec) -> Expr {
    create_value_expr(1, bitvec)
}

pub fn create_minus_one_expr(bitvec: &Bitvec) -> Expr {
    create_arith_neg_expr(create_value_expr(1, bitvec))
}
