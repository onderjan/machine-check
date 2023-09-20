use crate::btor2::{rref::Rref, sort::Sort};

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

    pub fn create_expression(&self, _result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
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
            BiOpType::Rol => todo!(),
            BiOpType::Ror => todo!(),
            BiOpType::Sll => Ok(quote!(!(::machine_check_types::Sll::sll(#a_tokens, #b_tokens)))),
            BiOpType::Sra => Ok(quote!(!(::machine_check_types::Sra::sra(#a_tokens, #b_tokens)))),
            BiOpType::Srl => Ok(quote!(!(::machine_check_types::Srl::srl(#a_tokens, #b_tokens)))),
            BiOpType::Add => Ok(quote!((#a_tokens) + (#b_tokens))),
            BiOpType::Mul => todo!(),
            BiOpType::Sdiv => todo!(),
            BiOpType::Udiv => todo!(),
            BiOpType::Smod => todo!(),
            BiOpType::Srem => todo!(),
            BiOpType::Urem => todo!(),
            BiOpType::Sub => Ok(quote!((#a_tokens) - (#b_tokens))),
            BiOpType::Saddo => todo!(),
            BiOpType::Uaddo => todo!(),
            BiOpType::Sdivo => todo!(),
            BiOpType::Udivo => todo!(),
            BiOpType::Smulo => todo!(),
            BiOpType::Umulo => todo!(),
            BiOpType::Ssubo => todo!(),
            BiOpType::Usubo => todo!(),
            BiOpType::Concat => todo!(),
            BiOpType::Read => {
                // a is the array, b is the index
                Ok(quote!(::machine_check_types::MachineArray::read(&(#a_tokens), #b_tokens)))
            }
        }
    }
}
