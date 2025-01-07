use btor2rs::{
    id::Rnid,
    op::{BiOp, BiOpType},
};
use syn::{parse_quote, Expr, Stmt};

use crate::translate::btor2::{util::create_rnid_expr, Error};

use super::{constant::create_value_expr, uni::create_bit_not, NodeTranslator};

impl NodeTranslator<'_> {
    pub fn bi_op_expr(&mut self, op: &BiOp) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        let a_expr = create_rnid_expr(op.a);
        let b_expr = create_rnid_expr(op.b);
        Ok((
            match op.ty {
                BiOpType::Iff => return self.create_eq(a_expr, b_expr),
                BiOpType::Implies => {
                    // a implies b = !a | b (on bit type)
                    let not_a = create_bit_not(a_expr);
                    create_bit_or(not_a, b_expr)
                }
                BiOpType::Eq => return self.create_eq(a_expr, b_expr),
                BiOpType::Neq => return self.create_ne(a_expr, b_expr),
                // lesser is implemented
                BiOpType::Ult => return self.create_comparison(op.a, op.b, false, false),
                BiOpType::Ulte => return self.create_comparison(op.a, op.b, false, true),
                BiOpType::Slt => return self.create_comparison(op.a, op.b, true, false),
                BiOpType::Slte => return self.create_comparison(op.a, op.b, true, true),
                // implement greater using lesser by flipping the operands
                BiOpType::Ugt => return self.create_comparison(op.b, op.a, false, false),
                BiOpType::Ugte => return self.create_comparison(op.b, op.a, false, true),
                BiOpType::Sgt => return self.create_comparison(op.b, op.a, true, false),
                BiOpType::Sgte => return self.create_comparison(op.b, op.a, true, true),
                BiOpType::And => create_bit_and(a_expr, b_expr),
                BiOpType::Nand => create_bit_not(create_bit_and(a_expr, b_expr)),
                BiOpType::Or => create_bit_or(a_expr, b_expr),
                BiOpType::Nor => create_bit_not(create_bit_or(a_expr, b_expr)),
                BiOpType::Xor => create_bit_xor(a_expr, b_expr),
                BiOpType::Xnor => create_bit_not(create_bit_xor(a_expr, b_expr)),
                BiOpType::Rol => return Err(Error::NotImplemented(op.ty.to_string())),
                BiOpType::Ror => return Err(Error::NotImplemented(op.ty.to_string())),
                BiOpType::Sll => create_logic_shl(a_expr, b_expr),
                BiOpType::Srl => self.shr_expr(op.a, op.b, false)?,
                BiOpType::Sra => self.shr_expr(op.a, op.b, true)?,
                BiOpType::Add => create_add(a_expr, b_expr),
                BiOpType::Sub => create_sub(a_expr, b_expr),
                BiOpType::Mul => create_mul(a_expr, b_expr),
                BiOpType::Udiv => self.divrem_expr(op.a, op.b, false, false)?,
                BiOpType::Urem => self.divrem_expr(op.a, op.b, false, true)?,
                BiOpType::Sdiv => self.divrem_expr(op.a, op.b, true, false)?,
                BiOpType::Srem => self.divrem_expr(op.a, op.b, true, true)?,
                BiOpType::Smod => return Err(Error::NotImplemented(op.ty.to_string())),
                BiOpType::Saddo
                | BiOpType::Uaddo
                | BiOpType::Sdivo
                | BiOpType::Udivo
                | BiOpType::Smulo
                | BiOpType::Umulo
                | BiOpType::Ssubo
                | BiOpType::Usubo => return Err(Error::NotImplemented(op.ty.to_string())),
                BiOpType::Concat => return self.concat_expr(op, op.a, op.b),
                BiOpType::Read => return Err(Error::NotImplemented(op.ty.to_string())),
            },
            vec![],
        ))
    }

    fn concat_expr(&mut self, op: &BiOp, a: Rnid, b: Rnid) -> Result<(Expr, Vec<Stmt>), Error> {
        let a_length = self.get_nid_bitvec(a.nid())?.length.get();
        let b_length = self.get_nid_bitvec(b.nid())?.length.get();

        let a_expr = create_rnid_expr(op.a);
        let b_expr = create_rnid_expr(op.b);

        // a is the higher, b is the lower
        let result_sort = self.get_bitvec(op.sid)?;
        let result_length = result_sort.length.get();
        let shift_length_expr = create_value_expr(b_length.into(), result_sort);

        // do unsigned extension of both to result type
        let (a_uext, mut stmts) = self.create_uext(a_expr, a_length, result_length)?;
        let (b_uext, b_stmts) = self.create_uext(b_expr, b_length, result_length)?;
        stmts.extend(b_stmts);

        // shift a left by length of b
        let a_uext_sll: Expr = create_logic_shl(a_uext, shift_length_expr);
        // bit-or together
        Ok((create_bit_or(a_uext_sll, b_uext), stmts))
    }

    pub(super) fn divrem_expr(
        &self,
        a: Rnid,
        b: Rnid,
        signed: bool,
        rem: bool,
    ) -> Result<Expr, Error> {
        let (a_expr, b_expr, length) = self.same_length_exprs(a, b)?;

        let ty: syn::Path = if signed {
            parse_quote!(::machine_check::<Signed<#length>>)
        } else {
            parse_quote!(::machine_check::<Unsigned<#length>>)
        };

        let a_expr: Expr = parse_quote!(::std::convert::Into::<#ty>::into(#a_expr));
        let b_expr: Expr = parse_quote!(::std::convert::Into::<#ty>::into(#b_expr));

        let op_result: Expr = if rem {
            parse_quote!(#a_expr % #b_expr)
        } else {
            parse_quote!(#a_expr / #b_expr)
        };

        Ok(parse_quote!(::std::convert::Into::<::machine_check::<Bitvector<#length>>>(#op_result)))
    }

    pub(super) fn shr_expr(&self, a: Rnid, b: Rnid, signed: bool) -> Result<Expr, Error> {
        let (a_expr, b_expr, length) = self.same_length_exprs(a, b)?;
        self.shr_expr_from_exprs(a_expr, b_expr, length, signed)
    }

    pub(super) fn shr_expr_from_exprs(
        &self,
        a_expr: Expr,
        b_expr: Expr,
        length: u32,
        signed: bool,
    ) -> Result<Expr, Error> {
        let ty: syn::Path = if signed {
            parse_quote!(::machine_check::Signed<#length>)
        } else {
            parse_quote!(::machine_check::Unsigned<#length>)
        };

        let a_expr: Expr = parse_quote!(::std::convert::Into::<#ty>::into(#a_expr));
        let b_expr: Expr = parse_quote!(::std::convert::Into::<#ty>::into(#b_expr));
        Ok(
            parse_quote!(::std::convert::Into::<::machine_check::Bitvector<#length>>::into(#a_expr >> #b_expr)),
        )
    }

    fn same_length_exprs(&self, a: Rnid, b: Rnid) -> Result<(Expr, Expr, u32), Error> {
        let a_length = self.get_nid_bitvec(a.nid())?.length.get();
        let b_length = self.get_nid_bitvec(b.nid())?.length.get();

        if a_length != b_length {
            return Err(Error::InvalidBiOp(a.nid(), b.nid()));
        }

        let a_expr = create_rnid_expr(a);
        let b_expr = create_rnid_expr(b);

        Ok((a_expr, b_expr, a_length))
    }

    fn create_condition(
        &mut self,
        cond_expr: Expr,
    ) -> Result<(Expr, Vec<Stmt>), Error> {
        let temp_ident = self.create_next_temporary();
        let local_stmt: Stmt = parse_quote!(let #temp_ident: ::machine_check::Bitvector<1>;);
        let if_stmt: Stmt = parse_quote!(if #cond_expr { 
            #temp_ident = ::machine_check::Bitvector::<1>::new(1); 
        } else {  
            #temp_ident = ::machine_check::Bitvector::<1>::new(0); 
        };);

        let result_expr: Expr = parse_quote!(#temp_ident);

        Ok((result_expr, vec![local_stmt, if_stmt]))
    }

    // equality
    pub(crate) fn create_eq(
        &mut self,
        a_expr: Expr,
        b_expr: Expr,
    ) -> Result<(Expr, Vec<Stmt>), Error> {
    self.create_condition(parse_quote!(#a_expr == #b_expr))
    }
    pub(crate) fn create_ne(
        &mut self,
        a_expr: Expr,
        b_expr: Expr,
    ) -> Result<(Expr, Vec<Stmt>), Error> {
    self.create_condition(parse_quote!(#a_expr != #b_expr))
    }
    
// comparison
    pub(crate) fn create_comparison(
        &mut self,
        a: Rnid, b: Rnid,
        signed: bool,
        can_be_equal: bool,
    ) -> Result<(Expr, Vec<Stmt>), Error> {
        let (a_expr, b_expr, length) = self.same_length_exprs(a, b)?;
        let ty: syn::Path = if signed {
            parse_quote!(::machine_check::Signed<#length>)
        } else {
            parse_quote!(::machine_check::Unsigned<#length>)
        };

        let a_into_ident = self.create_next_temporary();
        let a_into_stmt: Stmt = parse_quote!(let #a_into_ident: #ty = ::std::convert::Into::<#ty>::into(#a_expr););
        
        let b_into_ident = self.create_next_temporary();
        let b_into_stmt: Stmt = parse_quote!(let #b_into_ident: #ty = ::std::convert::Into::<#ty>::into(#b_expr););
        
        let cond_expr = if can_be_equal {
            parse_quote!(#a_into_ident <= #b_into_ident)
        } else {
            parse_quote!(#a_into_ident < #b_into_ident)
        };

        let mut stmts = vec![a_into_stmt, b_into_stmt];
        let (result_expr, condition_stmts) = self.create_condition(cond_expr)?;
        stmts.extend(condition_stmts);
        Ok((result_expr, stmts))
    }

}


// bitwise
pub(super) fn create_bit_and(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr & #b_expr))
}

pub(super) fn create_bit_or(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr | #b_expr))
}

pub(super) fn create_bit_xor(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr ^ #b_expr))
}

// arith
pub(super) fn create_add(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr + #b_expr))
}

pub(super) fn create_sub(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr - #b_expr))
}

pub(super) fn create_mul(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr * #b_expr))
}

// shift
pub(super) fn create_logic_shl(a_expr: Expr, b_expr: Expr) -> Expr {
    parse_quote!((#a_expr << #b_expr))
}
