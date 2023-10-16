//! Btor2 standard nodes.
//!
//! The type structure for nodes was chosen so that the nodes that have common fields
//! are combined. For example, `Init` and `Next` both specify an assignment
//! to a state at some point in time. They have the same fields as well,
//! so they are subtypes of `Temporal`.

use crate::{
    id::{Nid, Rnid, Sid},
    line::LineError,
    op::{BiOp, BiOpType, ExtOp, ExtOpType, SliceOp, TriOp, TriOpType, UniOp, UniOpType},
    util::parse_nid,
    util::parse_sid,
    util::parse_u32,
};

/// Btor2 node.
///
/// The result node id is stored outside of this structure, which is only concerned
///  about the line fragments following it that determine node type and arguments.
#[derive(Debug, Clone)]
pub enum Node {
    Const(Const),
    Source(Source),
    Drain(Drain),
    State(State),
    Temporal(Temporal),
    ExtOp(ExtOp),
    SliceOp(SliceOp),
    UniOp(UniOp),
    BiOp(BiOp),
    TriOp(TriOp),
    Justice(Justice),
}

/// Constant node.
///
/// As constants can have (almost) arbitrary lengths and the length is not known
/// without looking up the sort, the constant is not parsed by this crate
/// and just stored as a string.
#[derive(Debug, Clone)]
pub struct Const {
    /// Constant type.
    pub ty: ConstType,
    /// Result sort id.
    pub sid: Sid,
    /// Constant value stored as a string.
    pub value: String,
}

/// Constant type.
///
/// Binary, decimal, or hexadecimal.
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ConstType {
    /// Binary constant type.
    Const = 2,
    /// Decimal constant type.
    Constd = 10,
    /// Hexadecimal constant type.
    Consth = 16,
}

/// Source node type.Encompasses "input", "one", "ones", and "zero".
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SourceType {
    Input,
    One,
    Ones,
    Zero,
}

/// Source node type. Encompasses "input", "one", "ones", and "zero".
///
/// In essence, all of these are value sources in a sense, the "input" just
/// can take any value, while the others have a fixed value.
#[derive(Debug, Clone)]
pub struct Source {
    /// Source type.
    pub ty: SourceType,
    /// Source sort id.
    pub sid: Sid,
}

/// Drain node type. Encompasses "bad", "constraint", "fair", and "output".
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DrainType {
    Bad,
    Constraint,
    Fair,
    Output,
}

/// Drain node. Encompasses "bad", "constraint", "fair", and "output".
///
/// In essence, all of these are value drains in a sense, some just have
/// special verification behaviour.
#[derive(Debug, Clone)]
pub struct Drain {
    /// Drain type.
    pub ty: DrainType,
    /// Right-side node id to drain.
    pub rnid: Rnid,
}

/// State node.
#[derive(Debug, Clone)]
pub struct State {
    /// State sort id.
    pub sid: Sid,
}

/// Temporal node type. Encompasses "init" and "next".
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TemporalType {
    Init,
    Next,
}

/// Temporal node. Encompasses "init" and "next".
#[derive(Debug, Clone)]
pub struct Temporal {
    /// Temporal node type.
    pub ty: TemporalType,
    /// Result sort id.
    pub sid: Sid,
    /// State to apply the temporal assignment to.
    pub state: Nid,
    /// Value to assign in the corresponding temporal frame.
    pub value: Rnid,
}

/// Justice node.
#[derive(Debug, Clone)]
pub struct Justice {
    /// Vector of justice properties.
    pub nids: Vec<Rnid>,
}

impl Node {
    /// Return result sort id if it is available.
    ///
    /// Drain and justice nodes do not have a result sort id.
    pub fn get_result_sid(&self) -> Option<Sid> {
        Some(match self {
            Node::Source(n) => n.sid,
            Node::Const(n) => n.sid,
            Node::State(n) => n.sid,
            Node::ExtOp(n) => n.sid,
            Node::SliceOp(n) => n.sid,
            Node::UniOp(n) => n.sid,
            Node::BiOp(n) => n.sid,
            Node::TriOp(n) => n.sid,
            Node::Temporal(n) => n.sid,
            Node::Drain(_) => return None,
            Node::Justice(_) => return None,
        })
    }

    pub(crate) fn try_parse<'a>(
        second: &str,
        mut split: impl Iterator<Item = &'a str>,
    ) -> Result<Option<Node>, LineError> {
        // const
        if let Ok(ty) = ConstType::try_from(second) {
            let node = parse_const_node(ty, &mut split)?;
            return Ok(Some(node));
        }

        // source
        if let Ok(ty) = SourceType::try_from(second) {
            let sid = parse_sid(&mut split)?;
            return Ok(Some(Node::Source(Source { ty, sid })));
        }

        // drain
        if let Ok(ty) = DrainType::try_from(second) {
            let rnid = parse_rnid(&mut split)?;
            return Ok(Some(Node::Drain(Drain { ty, rnid })));
        }

        // temporal
        if let Ok(ty) = TemporalType::try_from(second) {
            let sid = parse_sid(&mut split)?;
            let state = parse_nid(&mut split)?;
            let value = parse_rnid(&mut split)?;
            return Ok(Some(Node::Temporal(Temporal {
                ty,
                sid,
                state,
                value,
            })));
        }

        // unary operations
        if let Ok(ty) = UniOpType::try_from(second) {
            let sid = parse_sid(&mut split)?;
            let a = parse_rnid(&mut split)?;
            return Ok(Some(Node::UniOp(UniOp { sid, ty, a })));
        }

        // binary operations
        if let Ok(ty) = BiOpType::try_from(second) {
            let sid = parse_sid(&mut split)?;
            let a = parse_rnid(&mut split)?;
            let b = parse_rnid(&mut split)?;
            return Ok(Some(Node::BiOp(BiOp { sid, ty, a, b })));
        }

        // ternary operations
        if let Ok(ty) = TriOpType::try_from(second) {
            let sid = parse_sid(&mut split)?;
            let a = parse_rnid(&mut split)?;
            let b = parse_rnid(&mut split)?;
            let c = parse_rnid(&mut split)?;
            return Ok(Some(Node::TriOp(TriOp { sid, ty, a, b, c })));
        }

        // extension
        if let Ok(ty) = ExtOpType::try_from(second) {
            let sid = parse_sid(&mut split)?;
            let a = parse_rnid(&mut split)?;
            let length = parse_u32(&mut split)?;
            return Ok(Some(Node::ExtOp(ExtOp { sid, ty, a, length })));
        }

        // other node types
        Ok(Some(match second {
            "slice" => {
                let sid = parse_sid(&mut split)?;
                let a = parse_rnid(&mut split)?;
                let upper_bit = parse_u32(&mut split)?;
                let lower_bit = parse_u32(&mut split)?;

                if upper_bit < lower_bit {
                    return Err(LineError::InvalidSlice);
                }
                Node::SliceOp(SliceOp {
                    sid,
                    a,
                    upper_bit,
                    lower_bit,
                })
            }
            "state" => {
                let sid = parse_sid(&mut split)?;
                Node::State(State { sid })
            }
            "justice" => {
                let num = parse_u32(&mut split)?;
                let mut vec = Vec::new();
                for _ in 0..num {
                    vec.push(parse_rnid(&mut split)?);
                }
                Node::Justice(Justice { nids: vec })
            }
            _ => {
                return Ok(None);
            }
        }))
    }
}

fn parse_const_node<'a>(
    ty: ConstType,
    split: &mut impl Iterator<Item = &'a str>,
) -> Result<Node, LineError> {
    let sid = parse_sid(split)?;
    let str = split.next().ok_or(LineError::MissingConstant)?;
    Ok(Node::Const(Const {
        ty,
        sid,
        value: String::from(str),
    }))
}

fn parse_rnid<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<Rnid, LineError> {
    let str = split.next().ok_or(LineError::MissingRnid)?;
    Rnid::try_from_str(str)
}
