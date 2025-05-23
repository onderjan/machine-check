use proc_macro2::Span;

use crate::{
    abstr::ZAbstr,
    wir::{
        WBlock, WCallArg, WExpr, WExprCall, WIdent, WIfCondition, WMckNew, WPath, WPathSegment,
        WStmt, WStmtAssign,
    },
};

pub struct IdentRenamer {
    prefix: String,
    make_super: bool,
}

impl IdentRenamer {
    pub fn new(prefix: String, make_super: bool) -> Self {
        Self { prefix, make_super }
    }

    pub fn visit_block(&self, block: &mut WBlock<ZAbstr>) {
        for stmt in &mut block.stmts {
            self.visit_stmt(stmt);
        }
    }

    pub fn visit_stmt(&self, stmt: &mut WStmt<ZAbstr>) {
        match stmt {
            WStmt::Assign(stmt) => self.visit_assign(stmt),
            WStmt::If(stmt) => {
                match &mut stmt.condition {
                    WIfCondition::Ident(condition_ident) => {
                        self.visit_ident(&mut condition_ident.ident)
                    }
                    WIfCondition::Literal(_) => {}
                };
                self.visit_block(&mut stmt.then_block);
                self.visit_block(&mut stmt.else_block);
            }
        }
    }

    pub fn visit_assign(&self, stmt: &mut WStmtAssign<ZAbstr>) {
        self.visit_ident(&mut stmt.left);
        self.visit_expr(&mut stmt.right);
    }

    pub fn visit_expr(&self, expr: &mut WExpr<WExprCall>) {
        match expr {
            WExpr::Move(ident) => self.visit_ident(ident),
            WExpr::Call(expr) => self.visit_call(expr),
            WExpr::Field(expr) => {
                self.visit_ident(&mut expr.base);
            }
            WExpr::Struct(expr) => {
                self.visit_path(&mut expr.type_path);
                for (_field_name, field_value) in &mut expr.fields {
                    self.visit_ident(field_value);
                }
            }
            WExpr::Reference(expr) => match expr {
                crate::wir::WExprReference::Ident(ident) => self.visit_ident(ident),
                crate::wir::WExprReference::Field(expr) => {
                    self.visit_ident(&mut expr.base);
                }
            },
            WExpr::Lit(_) => {}
        }
    }

    pub fn visit_call(&self, expr: &mut WExprCall) {
        match expr {
            WExprCall::Call(call) => {
                self.visit_path(&mut call.fn_path);
                for arg in &mut call.args {
                    match arg {
                        WCallArg::Ident(ident) => {
                            self.visit_ident(ident);
                        }
                        WCallArg::Literal(_) => {}
                    }
                }
            }
            WExprCall::MckUnary(call) => self.visit_ident(&mut call.operand),
            WExprCall::MckBinary(call) => {
                self.visit_ident(&mut call.a);
                self.visit_ident(&mut call.b);
            }
            WExprCall::MckExt(call) => {
                self.visit_ident(&mut call.from);
            }
            WExprCall::MckNew(call) => match call {
                WMckNew::Bitvector(_, _) => {}
                WMckNew::BitvectorArray(_, element) => self.visit_ident(element),
            },
            WExprCall::StdClone(ident) => {
                self.visit_ident(ident);
            }
            WExprCall::ArrayRead(call) => {
                self.visit_ident(&mut call.base);
                self.visit_ident(&mut call.index);
            }
            WExprCall::ArrayWrite(call) => {
                self.visit_ident(&mut call.base);
                self.visit_ident(&mut call.index);
                self.visit_ident(&mut call.right);
            }
            WExprCall::Phi(ident_a, ident_b) => {
                self.visit_ident(ident_a);
                self.visit_ident(ident_b);
            }
            WExprCall::PhiTaken(ident) => {
                self.visit_ident(ident);
            }
            WExprCall::PhiMaybeTaken(maybe_taken) => {
                self.visit_ident(&mut maybe_taken.taken);
                self.visit_ident(&mut maybe_taken.condition);
            }
            WExprCall::PhiNotTaken => {}
            WExprCall::PhiUninit => {}
        }
    }

    pub fn visit_ident(&self, ident: &mut WIdent) {
        *ident = ident.mck_prefixed(&self.prefix);
    }

    pub fn visit_path(&self, path: &mut WPath) {
        if !path.leading_colon && self.make_super {
            path.segments.insert(
                0,
                WPathSegment {
                    ident: WIdent::new(String::from("super"), Span::call_site()),
                },
            );
        }
    }
}
