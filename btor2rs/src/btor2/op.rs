use std::collections::BTreeMap;

use crate::btor2::node::Const;

use super::{
    id::{FlippableNid, Nid},
    node::Node,
    sort::Sort,
};
use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExtOp {
    a: FlippableNid,
    extension_size: usize,
    signed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SliceOp {
    a: FlippableNid,
    low_bit: usize,
    high_bit: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UniOpType {
    Not,
    Inc,
    Dec,
    Neg,
    Redand,
    Redor,
    Redxor,
}

impl TryFrom<&str> for UniOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "not" => Ok(UniOpType::Not),
            "inc" => Ok(UniOpType::Inc),
            "dec" => Ok(UniOpType::Dec),
            "neg" => Ok(UniOpType::Neg),
            "redand" => Ok(UniOpType::Redand),
            "redor" => Ok(UniOpType::Redor),
            "redxor" => Ok(UniOpType::Redxor),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniOp {
    op_type: UniOpType,
    a: FlippableNid,
}

impl UniOp {
    pub fn try_new(
        result_sort: &Sort,
        op_type: UniOpType,
        a: FlippableNid,
    ) -> Result<UniOp, anyhow::Error> {
        // TODO: match types once arrays are supported
        Ok(UniOp { op_type, a })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let Sort::Bitvec(bitvec_length) = result_sort;
        match self.op_type {
            UniOpType::Not => Ok(quote!(!(#a_ident))),
            UniOpType::Inc => {
                let one = Const::new(false, 1).create_tokens(*bitvec_length);
                Ok(quote!((#a_ident) + (#one)))
            }
            UniOpType::Dec => {
                let one = Const::new(false, 1).create_tokens(*bitvec_length);
                Ok(quote!((#a_ident) - (#one)))
            }
            UniOpType::Neg => Ok(quote!(-(#a_ident))),
            UniOpType::Redand => todo!(),
            UniOpType::Redor => todo!(),
            UniOpType::Redxor => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

impl TryFrom<&str> for BiOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            // Boolean
            "iff" => Ok(BiOpType::Iff),
            "implies" => Ok(BiOpType::Implies),
            // (dis)equality
            "eq" => Ok(BiOpType::Eq),
            "neq" => Ok(BiOpType::Neq),
            // (un)signed equality
            "sgt" => Ok(BiOpType::Sgt),
            "ugt" => Ok(BiOpType::Ugt),
            "sgte" => Ok(BiOpType::Sgte),
            "ugte" => Ok(BiOpType::Ugte),
            "slt" => Ok(BiOpType::Slt),
            "ult" => Ok(BiOpType::Ult),
            "slte" => Ok(BiOpType::Slte),
            "ulte" => Ok(BiOpType::Ulte),
            // bitwise
            "and" => Ok(BiOpType::And),
            "nand" => Ok(BiOpType::Nand),
            "nor" => Ok(BiOpType::Nor),
            "or" => Ok(BiOpType::Or),
            "xnor" => Ok(BiOpType::Xnor),
            "xor" => Ok(BiOpType::Xor),
            // rotate
            "rol" => Ok(BiOpType::Rol),
            "ror" => Ok(BiOpType::Ror),
            // shift
            "sll" => Ok(BiOpType::Sll),
            "sra" => Ok(BiOpType::Sra),
            "srl" => Ok(BiOpType::Srl),
            // arithmetic
            "add" => Ok(BiOpType::Add),
            "mul" => Ok(BiOpType::Mul),
            "sdiv" => Ok(BiOpType::Sdiv),
            "udiv" => Ok(BiOpType::Udiv),
            "smod" => Ok(BiOpType::Smod),
            "srem" => Ok(BiOpType::Srem),
            "urem" => Ok(BiOpType::Urem),
            "sub" => Ok(BiOpType::Sub),
            // overflow
            "saddo" => Ok(BiOpType::Saddo),
            "uaddo" => Ok(BiOpType::Uaddo),
            "sdivo" => Ok(BiOpType::Sdivo),
            "udivo" => Ok(BiOpType::Udivo),
            "smulo" => Ok(BiOpType::Smulo),
            "umulo" => Ok(BiOpType::Umulo),
            "ssubo" => Ok(BiOpType::Ssubo),
            "usubo" => Ok(BiOpType::Usubo),
            // concatenation
            "concat" => Ok(BiOpType::Concat),
            // array read
            "read" => Ok(BiOpType::Read),
            _ => Err(()),
        }
    }
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
        // TODO: match types once arrays are supported
        match op_type {
            BiOpType::Eq | BiOpType::Iff => {
                let Sort::Bitvec(bitvec_length) = result_sort;
                if *bitvec_length != 1 {
                    return Err(anyhow!("Expected one-bit bitvec"));
                }
            }
            _ => (),
        }
        Ok(BiOp { op_type, a, b })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let b_ident = self.b.create_tokens("node");
        let Sort::Bitvec(bitvec_length) = result_sort;
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

impl TryFrom<&str> for TriOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "ite" => Ok(TriOpType::Ite),
            "write" => Ok(TriOpType::Write),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TriOp {
    op_type: TriOpType,
    a: FlippableNid,
    b: FlippableNid,
    c: FlippableNid,
}

impl TriOp {
    pub fn try_new(
        result_sort: &Sort,
        op_type: TriOpType,
        a: FlippableNid,
        b: FlippableNid,
        c: FlippableNid,
    ) -> Result<TriOp, anyhow::Error> {
        // TODO: match types once arrays are supported
        Ok(TriOp { op_type, a, b, c })
    }

    pub fn create_expression(
        &self,
        _result_sort: &Sort,
        nodes: &BTreeMap<Nid, Node>,
    ) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let b_ident = self.b.create_tokens("node");
        let c_ident = self.c.create_tokens("node");
        match self.op_type {
            TriOpType::Ite => {
                // to avoid control flow, convert condition to bitmask
                let then_branch = &self.b;
                let Some(then_node) = nodes.get(&then_branch.nid) else {
                    return Err(anyhow!("Unknown then branch nid {}", then_branch.nid));
                };
                let Sort::Bitvec(bitvec_length) = then_node.result_sort;
                let condition_mask =
                    quote!(::machine_check_types::Sext::<#bitvec_length>::sext(#a_ident));
                let neg_condition_mask =
                    quote!(::machine_check_types::Sext::<#bitvec_length>::sext(!(#a_ident)));

                Ok(quote!((#b_ident & #condition_mask) | (#c_ident & #neg_condition_mask)))
            }
            TriOpType::Write => todo!(),
        }
    }
}
