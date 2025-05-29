use super::DualInterval;
use crate::{
    bitvector::concrete::{ConcreteBitvector, SignlessInterval},
    misc::MetaEq,
};

use std::fmt::{Debug, Display};

impl<const W: u32> MetaEq for DualInterval<W> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.near_half == other.near_half && self.far_half == other.far_half
    }
}

impl<const W: u32> DualInterval<W> {
    pub fn from_value(value: ConcreteBitvector<W>) -> Self {
        Self {
            near_half: SignlessInterval::from_value(value),
            far_half: SignlessInterval::from_value(value),
        }
    }

    pub fn contains_value(&self, value: &ConcreteBitvector<W>) -> bool {
        self.near_half.contains_value(value) || self.far_half.contains_value(value)
    }

    pub fn contains(&self, other: &Self) -> bool {
        println!("Testing if {:?} contains {:?}", self, other);
        if other.near_half == other.far_half {
            let tested_half = other.near_half;
            if tested_half.is_sign_bit_set() {
                self.far_half.contains(&other.far_half)
            } else {
                self.near_half.contains(&other.near_half)
            }
        } else {
            self.near_half.contains(&other.near_half) && self.far_half.contains(&other.far_half)
        }
    }

    pub fn concrete_value(&self) -> Option<ConcreteBitvector<W>> {
        if self.near_half == self.far_half {
            return self.near_half.concrete_value();
        }
        None
    }

    pub fn concrete_join(self, value: ConcreteBitvector<W>) -> Self {
        let value_sign_bit_set = value.is_sign_bit_set();
        let value = SignlessInterval::from_value(value);

        if self.near_half == self.far_half {
            if value_sign_bit_set == self.near_half.is_sign_bit_set() {
                // join to both halves
                Self {
                    near_half: self.near_half.union(value),
                    far_half: self.far_half.union(value),
                }
            } else {
                // we have to make a new half from the value
                if value_sign_bit_set {
                    Self {
                        near_half: self.near_half,
                        far_half: value,
                    }
                } else {
                    Self {
                        near_half: value,
                        far_half: self.far_half,
                    }
                }
            }
        } else if value_sign_bit_set {
            // join to far half
            Self {
                near_half: self.near_half,
                far_half: self.far_half.union(value),
            }
        } else {
            // join to near half
            Self {
                near_half: self.near_half.union(value),
                far_half: self.far_half,
            }
        }
    }

    pub fn all_with_length_iter() -> impl Iterator<Item = Self> {
        let only_near_half_result = SignlessInterval::all_with_length_iter(false)
            .map(|near_half| Self::from_opt_halves(Some(near_half), None));
        let only_far_half_result = SignlessInterval::all_with_length_iter(true)
            .map(|far_half| Self::from_opt_halves(None, Some(far_half)));

        let near_half_iter = SignlessInterval::<W>::all_with_length_iter(false);
        let both_halves_result = near_half_iter.flat_map(|near_half| {
            let far_half_iter = SignlessInterval::<W>::all_with_length_iter(true);
            far_half_iter
                .map(move |far_half| Self::from_opt_halves(Some(near_half), Some(far_half)))
        });
        only_near_half_result
            .chain(only_far_half_result)
            .chain(both_halves_result)
    }
}

impl<const W: u32> Debug for DualInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.near_half == self.far_half {
            write!(f, "{}", self.near_half)
        } else {
            write!(f, "{} ⊔ {}", self.near_half, self.far_half)
        }
    }
}

impl<const W: u32> Display for DualInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
