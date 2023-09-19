use std::{
    num::Wrapping,
    ops::{Add, BitAnd, BitOr, BitXor, Neg, Not, Sub},
};

#[derive(Debug, Clone, Copy)]
pub struct MachineBitvector<const N: u32> {
    v: Wrapping<u64>,
}

const fn compute_mask(n: u32) -> Wrapping<u64> {
    if n == u64::BITS {
        return Wrapping(0u64.wrapping_sub(1u64));
    }
    let num_values = u64::checked_shl(1u64, n);
    if let Some(num_values) = num_values {
        Wrapping(num_values.wrapping_sub(1u64))
    } else {
        panic!("Too many bits for MachineU")
    }
}

impl<const N: u32> MachineBitvector<N> {
    fn w_new(value: Wrapping<u64>) -> Self {
        let mask = compute_mask(N);
        if (value & !mask) != Wrapping(0) {
            panic!("MachineU value {} does not fit into {} bits", value, N);
        }

        //println!("New {}-bitvector (mask {}): {}", N, mask, value);

        MachineBitvector { v: value }
    }

    pub fn new(value: u64) -> Self {
        Self::w_new(Wrapping(value))
    }
}

impl<const N: u32> Neg for MachineBitvector<N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::w_new((-self.v) & compute_mask(N))
    }
}

impl<const N: u32> Add for MachineBitvector<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v + rhs.v) & compute_mask(N))
    }
}

impl<const N: u32> Sub for MachineBitvector<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v - rhs.v) & compute_mask(N))
    }
}

impl<const N: u32> Not for MachineBitvector<N> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::w_new((!self.v) & compute_mask(N))
    }
}

impl<const N: u32> BitAnd for MachineBitvector<N> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v & rhs.v) & compute_mask(N))
    }
}

impl<const N: u32> BitOr for MachineBitvector<N> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v | rhs.v) & compute_mask(N))
    }
}

impl<const N: u32> BitXor for MachineBitvector<N> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::w_new((self.v ^ rhs.v) & compute_mask(N))
    }
}

impl<const N: u32> PartialEq for MachineBitvector<N> {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

impl<const N: u32> Eq for MachineBitvector<N> {}

pub trait TypedEq {
    type Output;

    fn typed_eq(self, rhs: Self) -> Self::Output;
}

impl<const N: u32> TypedEq for MachineBitvector<N> {
    type Output = MachineBitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        MachineBitvector::<1>::w_new(Wrapping((self == rhs) as u64))
    }
}

pub trait Uext<const M: u32> {
    type Output;

    fn uext(self) -> Self::Output;
}

impl<const N: u32, const M: u32> Uext<M> for MachineBitvector<N> {
    type Output = MachineBitvector<M>;

    fn uext(self) -> Self::Output {
        // only shorten if needed
        MachineBitvector::<M>::w_new(self.v & compute_mask(M))
    }
}

pub trait Sext<const M: u32> {
    type Output;

    fn sext(self) -> Self::Output;
}

impl<const N: u32, const M: u32> Sext<M> for MachineBitvector<N> {
    type Output = MachineBitvector<M>;

    fn sext(self) -> Self::Output {
        // shorten if needed
        let mut v = self.v & compute_mask(M);
        // copy sign bit where necessary
        if M > N {
            let num_sign_extend = M - N;
            let sign_masked = self.v & (Wrapping(1u64) << (N - 1) as usize);
            for i in 1..num_sign_extend + 1 {
                v |= sign_masked << i as usize;
            }
        }

        MachineBitvector::<M>::w_new(v)
    }
}

pub trait Lsl {
    type Output;

    fn lsl(self, amount: Self) -> Self::Output;
}

impl<const N: u32> Lsl for MachineBitvector<N> {
    type Output = Self;

    fn lsl(self, amount: Self) -> Self {
        if amount.v.0 >= N as u64 {
            // zero if the shift is too big
            MachineBitvector::w_new(Wrapping(0))
        } else {
            MachineBitvector::w_new(self.v << amount.v.0 as usize)
        }
    }
}

pub trait Lsr {
    type Output;

    fn lsr(self, amount: Self) -> Self::Output;
}

impl<const N: u32> Lsr for MachineBitvector<N> {
    type Output = Self;

    fn lsr(self, amount: Self) -> Self {
        if amount.v.0 >= N as u64 {
            // zero if the shift is too big
            MachineBitvector::w_new(Wrapping(0))
        } else {
            MachineBitvector::w_new(self.v >> amount.v.0 as usize)
        }
    }
}

pub trait Asr {
    type Output;

    fn asr(self, amount: Self) -> Self::Output;
}

impl<const N: u32> Asr for MachineBitvector<N> {
    type Output = Self;

    fn asr(self, amount: Self) -> Self {
        let sign_masked = self.v & (Wrapping(1u64) << (N - 1) as usize);
        if amount.v.0 >= N as u64 {
            // fill with sign bit if the shift is too big
            if sign_masked != Wrapping(0) {
                MachineBitvector::w_new(compute_mask(N))
            } else {
                MachineBitvector::w_new(Wrapping(0))
            }
        } else {
            // copy sign bit where necessary
            let mut v = self.v >> amount.v.0 as usize;
            for i in 0..amount.v.0 {
                v |= sign_masked >> i as usize;
            }

            MachineBitvector::w_new(v)
        }
    }
}
