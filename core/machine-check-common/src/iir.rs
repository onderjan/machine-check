use mck::{
    abstr::AbstractValue,
    refin::{Limit, RefinementValue},
};

use crate::iir::variable::IVarId;

type IAbstr = mck::misc::Interpretation<IVarId, AbstractValue>;
type IRefin = mck::misc::Interpretation<IVarId, RefinementValue>;

pub mod context;
pub mod description;
pub mod expr;
pub mod func;
pub mod path;
pub mod property;
pub mod stmt;
pub mod ty;
pub mod variable;

fn join_limited(abstr: &IAbstr, refin: &mut IRefin, var_id: IVarId, refin_value: RefinementValue) {
    let abstr_value = abstr.value(var_id);
    let refin_value = refin_value.limit(abstr_value);
    refin.join_value(var_id, refin_value);
}
