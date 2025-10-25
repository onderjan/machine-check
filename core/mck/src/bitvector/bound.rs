use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::bitvector::{compute_u64_mask, util::compute_u64_sign_bit_mask};

pub trait BitvectorBound: Clone + Copy + PartialEq + Eq + Hash + Debug {
    fn width(&self) -> u32;
    fn mask(&self) -> u64;
    fn sign_bit_mask(&self) -> u64;

    fn allowed(&self, value: u64) -> bool {
        value <= self.mask()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct RBound {
    width: u32,
}

impl RBound {
    pub fn new(width: u32) -> Self {
        Self { width }
    }
}

impl BitvectorBound for RBound {
    fn width(&self) -> u32 {
        self.width
    }

    fn mask(&self) -> u64 {
        compute_u64_mask(self.width)
    }

    fn sign_bit_mask(&self) -> u64 {
        compute_u64_sign_bit_mask(self.width)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct CBound<const W: u32>;

impl<const W: u32> BitvectorBound for CBound<W> {
    fn width(&self) -> u32 {
        W
    }
    fn mask(&self) -> u64 {
        compute_u64_mask(W)
    }

    fn sign_bit_mask(&self) -> u64 {
        compute_u64_sign_bit_mask(W)
    }
}
