use super::WIdent;

#[derive(Clone, Debug, Hash)]
pub struct WStdUnary {
    pub op: WStdUnaryOp,
    pub operand: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WStdBinary {
    pub op: WStdBinaryOp,
    pub a: WIdent,
    pub b: WIdent,
}

#[derive(Clone, Debug, Hash, strum::EnumString, strum::Display)]
pub enum WStdUnaryOp {
    #[strum(to_string = "::std::ops::Not::not")]
    Not,
    #[strum(to_string = "::std::ops::Neg::neg")]
    Neg,
}

#[derive(Clone, Debug, Hash, strum::EnumString, strum::Display)]
pub enum WStdBinaryOp {
    // bitwise
    #[strum(to_string = "::std::ops::BitAnd::bitand")]
    BitAnd,
    #[strum(to_string = "::std::ops::BitOr::bitor")]
    BitOr,
    #[strum(to_string = "::std::ops::BitXor::bitxor")]
    BitXor,
    // shifts
    #[strum(to_string = "::std::ops::Shl::shl")]
    Shl,
    #[strum(to_string = "::std::ops::Shr::shr")]
    Shr,
    // arithmetic
    #[strum(to_string = "::std::ops::Add::add")]
    Add,
    #[strum(to_string = "::std::ops::Sub::sub")]
    Sub,
    #[strum(to_string = "::std::ops::Mul::mul")]
    Mul,
    // equality
    #[strum(to_string = "::std::cmp::PartialEq::eq")]
    Eq,
    #[strum(to_string = "::std::cmp::PartialEq::ne")]
    Ne,
    // comparison
    #[strum(to_string = "::std::cmp::PartialOrd::lt")]
    Lt,
    #[strum(to_string = "::std::cmp::PartialOrd::le")]
    Le,
    #[strum(to_string = "::std::cmp::PartialOrd::gt")]
    Gt,
    #[strum(to_string = "::std::cmp::PartialOrd::ge")]
    Ge,
}

#[derive(Clone, Debug, Hash)]
pub struct WMckUnary {
    pub op: WMckUnaryOp,
    pub operand: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WMckBinary {
    pub op: WMckBinaryOp,
    pub a: WIdent,
    pub b: WIdent,
}

#[derive(Clone, Debug, Hash, strum::EnumString, strum::Display)]
pub enum WMckUnaryOp {
    #[strum(to_string = "::mck::forward::Bitwise::bit_not")]
    Not,
    #[strum(to_string = "::std::ops::Neg::neg")]
    Neg,
}

#[derive(Clone, Debug, Hash, strum::EnumString, strum::Display)]
pub enum WMckBinaryOp {
    // bitwise
    #[strum(to_string = "::mck::forward::Bitwise::bit_and")]
    BitAnd,
    #[strum(to_string = "::mck::forward::Bitwise::bit_or")]
    BitOr,
    #[strum(to_string = "::mck::forward::Bitwise::bit_xor")]
    BitXor,
    // shifts
    #[strum(to_string = "::mck::forward::HwShift::logic_shl")]
    LogicShl,
    #[strum(to_string = "::mck::forward::HwShift::logic_shr")]
    LogicShr,
    #[strum(to_string = "::mck::forward::HwShift::arith_shr")]
    ArithShr,
    // arithmetic
    #[strum(to_string = "::mck::forward::HwArith::add")]
    Add,
    #[strum(to_string = "::mck::forward::HwArith::sub")]
    Sub,
    #[strum(to_string = "::mck::forward::HwArith::mul")]
    Mul,
    // equality
    #[strum(to_string = "::mck::forward::TypedEq::eq")]
    Eq,
    #[strum(to_string = "::mck::forward::TypedEq::ne")]
    Ne,
    // comparison
    #[strum(to_string = "::mck::forward::TypedCmp::ult")]
    Ult,
    #[strum(to_string = "::mck::forward::TypedCmp::ule")]
    Ule,
    #[strum(to_string = "::mck::forward::TypedCmp::slt")]
    Slt,
    #[strum(to_string = "::mck::forward::TypedCmp::sle")]
    Sle,
}
