use crate::{
    parse_nid, parse_sid, parse_u32, BiOp, BiOpType, ExtOp, ExtOpType, Nid, Rnid, Sid, SliceOp,
    TriOp, TriOpType, UniOp, UniOpType,
};
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct Const {
    pub ty: ConstType,
    pub sid: Sid,
    pub string: String,
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ConstType {
    Const = 2,
    Constd = 10,
    Consth = 16,
}

#[derive(Debug, Clone)]
pub enum OpType {
    Output(Rnid),
    Const(Const),
    Bad(Rnid),
    Constraint(Rnid),
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DrainType {
    Bad,
    Constraint,
    Fair,
    Output,
}

#[derive(Debug, Clone)]
pub struct Drain {
    pub ty: DrainType,
    pub rnid: Rnid,
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TemporalType {
    Init,
    Next,
}

#[derive(Debug, Clone)]
pub struct Temporal {
    pub ty: TemporalType,
    pub sid: Sid,
    pub state: Nid,
    pub value: Rnid,
}

#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SourceType {
    Input,
    One,
    Ones,
    Zero,
}
#[derive(Debug, Clone)]
pub struct Source {
    pub ty: SourceType,
    pub sid: Sid,
}

#[derive(Debug, Clone)]
pub struct State {
    pub sid: Sid,
}

#[derive(Debug, Clone)]
pub enum Node {
    Source(Source),
    Const(Const),
    State(State),
    ExtOp(ExtOp),
    SliceOp(SliceOp),
    UniOp(UniOp),
    BiOp(BiOp),
    TriOp(TriOp),
    Temporal(Temporal),
    Drain(Drain),
    Justice(Vec<Rnid>),
}

impl Node {
    pub fn get_sid(&self) -> Option<Sid> {
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
    ) -> Result<Option<Node>, anyhow::Error> {
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

        // other operations
        Ok(Some(match second {
            // special operations
            "slice" => {
                let sid = parse_sid(&mut split)?;
                let a = parse_rnid(&mut split)?;
                let upper_bit = parse_u32(&mut split)?;
                let lower_bit = parse_u32(&mut split)?;

                if upper_bit < lower_bit {
                    return Err(anyhow!(
                        "Upper bit {} cannot be lower than lower bit {}",
                        upper_bit,
                        lower_bit
                    ));
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
                Node::Justice(vec)
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
) -> Result<Node, anyhow::Error> {
    let sid = parse_sid(split)?;
    let str = split
        .next()
        .ok_or_else(|| anyhow!("Expected the constant"))?;
    Ok(Node::Const(Const {
        ty,
        sid,
        string: String::from(str),
    }))
}

fn parse_rnid<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<Rnid, anyhow::Error> {
    let str = split.next().ok_or_else(|| anyhow!("Missing nid"))?;
    Rnid::try_from(str)
}
