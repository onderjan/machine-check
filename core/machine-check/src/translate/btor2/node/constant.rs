use btor2rs::Const;
use syn::{parse_quote, Expr};

use super::NodeTranslator;

impl<'a> NodeTranslator<'a> {
    pub fn const_expr(&self, value: &Const) -> Result<Expr, anyhow::Error> {
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
            parse_quote!((::mck::forward::HwArith::arith_neg(::mck::concr::Bitvector::<#bitvec_length>::new(#value))))
        } else {
            parse_quote!(::mck::concr::Bitvector::<#bitvec_length>::new(#value))
        })
    }
}
