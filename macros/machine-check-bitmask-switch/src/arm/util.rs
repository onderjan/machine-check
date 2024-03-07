use std::fmt::Display;

use num::BigUint;

pub enum MaskBit {
    Literal(bool),
    Variable(char),
    DontCare,
}

#[derive(Clone, Debug)]
pub struct CareValue {
    pub num_bits: u64,
    pub care: BigUint,
    pub value: BigUint,
}

impl Display for CareValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        // write higher bits first
        for k in (0..self.num_bits).rev() {
            let care_k = self.care.bit(k);
            let value_k = self.value.bit(k);
            let c = if care_k {
                if value_k {
                    "1"
                } else {
                    "0"
                }
            } else {
                "-"
            };
            write!(f, "{}", c)?;
        }

        write!(f, "\"")
    }
}

impl CareValue {
    pub fn intersects(&self, other: &Self) -> bool {
        // the number of bits must be the same
        if self.num_bits != other.num_bits {
            return false;
        }

        // return true exactly if there is no bit where both cares are 1 and values are different
        let considered_bits = self.care.bits().min(other.care.bits());
        for k in 0..considered_bits {
            if self.care.bit(k) && other.care.clone().bit(k) {
                // if the values are different, they do not intersect
                if self.value.bit(k) != other.value.bit(k) {
                    return false;
                }
            }
        }
        true
    }

    pub fn try_combine(&self, other: &Self) -> Option<Self> {
        // self and other must have the same number of bits and cares
        if self.num_bits != other.num_bits || self.care != other.care {
            return None;
        }
        let considered_bits = self.care.bits();

        // exactly one considered value bit must be different for us to combine them
        for k in 0..considered_bits {
            if self.value.bit(k) != other.value.clone().bit(k) {
                let mut self_remaining_value = self.value.clone();
                self_remaining_value.set_bit(k, false);
                let mut other_remaining_value = other.value.clone();
                other_remaining_value.set_bit(k, false);
                if self_remaining_value == other_remaining_value {
                    // combine self and other with don't-care in k-th position
                    let mut result_care = self.care.clone();
                    result_care.set_bit(k, false);

                    return Some(CareValue {
                        num_bits: self.num_bits,
                        care: result_care,
                        value: self_remaining_value,
                    });
                }
            }
        }

        // no considered value bit found
        None
    }

    pub fn num_care_bits(&self) -> u64 {
        self.care.count_ones()
    }
}
