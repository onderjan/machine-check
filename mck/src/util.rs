use std::num::Wrapping;

pub const fn compute_mask(length: u32) -> Wrapping<u64> {
    if length == 0 {
        return Wrapping(0);
    }
    if length == u64::BITS {
        // this would fail in checked shl,
        // but the mask is just full of ones
        return Wrapping(0u64.wrapping_sub(1u64));
    }
    let num_values = u64::checked_shl(1u64, length);
    if let Some(num_values) = num_values {
        Wrapping(num_values.wrapping_sub(1u64))
    } else {
        panic!("Too many bits to compute mask")
    }
}

pub const fn compute_sign_bit_mask(length: u32) -> Wrapping<u64> {
    if length == 0 {
        // return zero
        return Wrapping(0);
    }
    // the highest bit within mask (unless length is 0)
    Wrapping(1u64.wrapping_shl(length - 1))
}

pub fn is_sign_bit_set(value: Wrapping<u64>, length: u32) -> bool {
    value & compute_sign_bit_mask(length) != Wrapping(0)
}
