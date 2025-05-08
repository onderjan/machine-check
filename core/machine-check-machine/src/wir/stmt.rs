use super::{expr::WExpr, path::WIdent, ty::WType};

#[derive(Clone, Debug, Hash)]
pub struct WBlock {
    pub locals: Vec<WLocal>,
    pub stmts: Vec<WStmt>,
}

#[derive(Clone, Debug, Hash)]
pub enum WStmt {
    Assign(WStmtAssign),
    If(WStmtIf),
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtAssign {
    pub left_ident: WIdent,
    pub right_expr: WExpr,
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtIf {
    pub condition: WExpr,
    pub then_block: WBlock,
    pub else_block: WBlock,
}

#[derive(Clone, Debug, Hash)]
pub struct WLocal {
    pub ident: WIdent,
    pub original: WIdent,
    pub ty: Option<WType>,
}
