use crate::btor2::{id::FlippableNid, sort::Sort};

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
    a: FlippableNid,
    b: FlippableNid,
}

impl BiOp {
    pub fn try_new(
        result_sort: &Sort,
        op_type: BiOpType,
        a: FlippableNid,
        b: FlippableNid,
    ) -> Result<BiOp, anyhow::Error> {
        // check result type
        // TODO: check operand types
        match op_type {
            BiOpType::Iff
            | BiOpType::Implies
            | BiOpType::Eq
            | BiOpType::Neq
            | BiOpType::Sgt
            | BiOpType::Ugt
            | BiOpType::Sgte
            | BiOpType::Ugte
            | BiOpType::Slt
            | BiOpType::Ult
            | BiOpType::Slte
            | BiOpType::Ulte
            | BiOpType::Saddo
            | BiOpType::Uaddo
            | BiOpType::Sdivo
            | BiOpType::Udivo
            | BiOpType::Smulo
            | BiOpType::Umulo
            | BiOpType::Ssubo
            | BiOpType::Usubo => {
                let Sort::Bitvec(bitvec_length) = result_sort else {
                    return Err(anyhow!("Expected one-bit result, but have {}", result_sort));
                };
                if *bitvec_length != 1 {
                    return Err(anyhow!("Expected one-bit result, but have {}", result_sort));
                }
            }
            BiOpType::And
            | BiOpType::Nand
            | BiOpType::Nor
            | BiOpType::Or
            | BiOpType::Xnor
            | BiOpType::Xor
            | BiOpType::Rol
            | BiOpType::Ror
            | BiOpType::Sll
            | BiOpType::Sra
            | BiOpType::Srl
            | BiOpType::Add
            | BiOpType::Mul
            | BiOpType::Sdiv
            | BiOpType::Udiv
            | BiOpType::Smod
            | BiOpType::Srem
            | BiOpType::Urem
            | BiOpType::Sub => {
                let Sort::Bitvec(_) = result_sort else {
                    return Err(anyhow!("Expected bitvector result, but have {}", result_sort));
                };
            }
            BiOpType::Concat | BiOpType::Read => todo!(),
        }
        Ok(BiOp { op_type, a, b })
    }

    pub fn create_expression(&self, _result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let b_ident = self.b.create_tokens("node");
        match self.op_type {
            BiOpType::Iff => {
                Ok(quote!(::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident)))
            }
            BiOpType::Implies => Ok(quote!(!(#a_ident) | (#b_ident))),
            BiOpType::Eq => {
                Ok(quote!(::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident)))
            }
            BiOpType::Neq => {
                Ok(quote!(!(::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident))))
            }
            BiOpType::Sgt => todo!(),
            BiOpType::Ugt => todo!(),
            BiOpType::Sgte => todo!(),
            BiOpType::Ugte => todo!(),
            BiOpType::Slt => todo!(),
            BiOpType::Ult => todo!(),
            BiOpType::Slte => todo!(),
            BiOpType::Ulte => todo!(),
            BiOpType::And => Ok(quote!((#a_ident) & (#b_ident))),
            BiOpType::Nand => Ok(quote!(!((#a_ident) & (#b_ident)))),
            BiOpType::Nor => Ok(quote!(!((#a_ident) & (#b_ident)))),
            BiOpType::Or => Ok(quote!((#a_ident) | (#b_ident))),
            BiOpType::Xnor => Ok(quote!(!((#a_ident) ^ (#b_ident)))),
            BiOpType::Xor => Ok(quote!((#a_ident) ^ (#b_ident))),
            BiOpType::Rol => todo!(),
            BiOpType::Ror => todo!(),
            BiOpType::Sll => Ok(quote!(!(::machine_check_types::Sll::sll(#a_ident, #b_ident)))),
            BiOpType::Sra => Ok(quote!(!(::machine_check_types::Sra::sra(#a_ident, #b_ident)))),
            BiOpType::Srl => Ok(quote!(!(::machine_check_types::Srl::srl(#a_ident, #b_ident)))),
            BiOpType::Add => Ok(quote!((#a_ident) + (#b_ident))),
            BiOpType::Mul => todo!(),
            BiOpType::Sdiv => todo!(),
            BiOpType::Udiv => todo!(),
            BiOpType::Smod => todo!(),
            BiOpType::Srem => todo!(),
            BiOpType::Urem => todo!(),
            BiOpType::Sub => Ok(quote!((#a_ident) - (#b_ident))),
            BiOpType::Saddo => todo!(),
            BiOpType::Uaddo => todo!(),
            BiOpType::Sdivo => todo!(),
            BiOpType::Udivo => todo!(),
            BiOpType::Smulo => todo!(),
            BiOpType::Umulo => todo!(),
            BiOpType::Ssubo => todo!(),
            BiOpType::Usubo => todo!(),
            BiOpType::Concat => todo!(),
            BiOpType::Read => todo!(),
        }
    }
}
