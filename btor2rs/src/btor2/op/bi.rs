use crate::btor2::{node::Const, rref::Rref, sort::Sort};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

// derive Btor2 string representations, which are lower-case
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum BiOpType {
    // Boolean
    Iff,
    Implies,
    // (dis)equality
    Eq,
    Neq,
    // (un)signed equality
    Sgt,
    Ugt,
    Sgte,
    Ugte,
    Slt,
    Ult,
    Slte,
    Ulte,
    // bitwise
    And,
    Nand,
    Nor,
    Or,
    Xnor,
    Xor,
    // rotate
    Rol,
    Ror,
    // shift
    Sll,
    Sra,
    Srl,
    // arithmetic
    Add,
    Mul,
    Sdiv,
    Udiv,
    Smod,
    Srem,
    Urem,
    Sub,
    // overflow
    Saddo,
    Uaddo,
    Sdivo,
    Udivo,
    Smulo,
    Umulo,
    Ssubo,
    Usubo,
    // concatenation
    Concat,
    // array read
    Read,
}

#[derive(Debug, Clone)]
pub struct BiOp {
    op_type: BiOpType,
    a: Rref,
    b: Rref,
}

impl BiOp {
    pub fn new(op_type: BiOpType, a: Rref, b: Rref) -> BiOp {
        BiOp { op_type, a, b }
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_tokens = self.a.create_tokens("node");
        let b_tokens = self.b.create_tokens("node");
        match self.op_type {
            BiOpType::Iff => {
                Ok(quote!(::machine_check_types::TypedEq::typed_eq(#a_tokens, #b_tokens)))
            }
            BiOpType::Implies => Ok(quote!(!(#a_tokens) | (#b_tokens))),
            BiOpType::Eq => {
                Ok(quote!(::machine_check_types::TypedEq::typed_eq(#a_tokens, #b_tokens)))
            }
            BiOpType::Neq => {
                Ok(quote!(!(::machine_check_types::TypedEq::typed_eq(#a_tokens, #b_tokens))))
            }
            BiOpType::Sgt => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_sgt(#a_tokens, #b_tokens)))
            }
            BiOpType::Ugt => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_ugt(#a_tokens, #b_tokens)))
            }
            BiOpType::Sgte => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_sgte(#a_tokens, #b_tokens)))
            }
            BiOpType::Ugte => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_ugte(#a_tokens, #b_tokens)))
            }
            BiOpType::Slt => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_slt(#a_tokens, #b_tokens)))
            }
            BiOpType::Ult => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_ult(#a_tokens, #b_tokens)))
            }
            BiOpType::Slte => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_slte(#a_tokens, #b_tokens)))
            }
            BiOpType::Ulte => {
                Ok(quote!(::machine_check_types::TypedCmp::typed_ulte(#a_tokens, #b_tokens)))
            }
            BiOpType::And => Ok(quote!((#a_tokens) & (#b_tokens))),
            BiOpType::Nand => Ok(quote!(!((#a_tokens) & (#b_tokens)))),
            BiOpType::Nor => Ok(quote!(!((#a_tokens) & (#b_tokens)))),
            BiOpType::Or => Ok(quote!((#a_tokens) | (#b_tokens))),
            BiOpType::Xnor => Ok(quote!(!((#a_tokens) ^ (#b_tokens)))),
            BiOpType::Xor => Ok(quote!((#a_tokens) ^ (#b_tokens))),
            BiOpType::Rol => Err(anyhow!("Left rotation generation not implemented")),
            BiOpType::Ror => Err(anyhow!("Right rotation generation not implemented")),
            BiOpType::Sll => Ok(quote!(::machine_check_types::Sll::sll(#a_tokens, #b_tokens))),
            BiOpType::Sra => Ok(quote!(::machine_check_types::Sra::sra(#a_tokens, #b_tokens))),
            BiOpType::Srl => Ok(quote!(::machine_check_types::Srl::srl(#a_tokens, #b_tokens))),
            BiOpType::Add => Ok(quote!((#a_tokens) + (#b_tokens))),
            BiOpType::Mul => Ok(quote!((#a_tokens) * (#b_tokens))),
            BiOpType::Sdiv => Err(anyhow!("Signed division generation not implemented")),
            BiOpType::Udiv => Err(anyhow!("Unsigned division generation not implemented")),
            BiOpType::Smod => Err(anyhow!("Signed modulo generation not implemented")),
            BiOpType::Srem => Err(anyhow!("Signed remainder generation not implemented")),
            BiOpType::Urem => Err(anyhow!("Unsigned remainder generation not implemented")),
            BiOpType::Sub => Ok(quote!((#a_tokens) - (#b_tokens))),
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
                    return Err(anyhow!("Expected bitvec result, but have {}", result_sort));
                };
                let result_length = result_sort.length.get();

                // do unsigned extension of both to result type
                let a_uext = quote!(::machine_check_types::Uext::<#result_length>::uext(#a_tokens));
                let b_uext = quote!(::machine_check_types::Uext::<#result_length>::uext(#b_tokens));

                // shift a left by length of b
                let Sort::Bitvec(b_sort) = &self.b.sort else {
                    return Err(anyhow!("Expected bitvec second parameter, but have {}", self.b.sort));
                };
                let b_length = b_sort.length.get();

                let sll_const = Const::new(false, b_length as u64);
                let sll_tokens = sll_const.create_tokens(result_sort);
                let a_uext_sll = quote!(::machine_check_types::Sll::sll(#a_uext, #sll_tokens));

                // bit-or together
                Ok(quote!((#a_uext_sll) | (#b_uext)))
            }
            BiOpType::Read => {
                // a is the array, b is the index
                Ok(quote!(::machine_check_types::MachineArray::read(&(#a_tokens), #b_tokens)))
            }
        }
    }
}
