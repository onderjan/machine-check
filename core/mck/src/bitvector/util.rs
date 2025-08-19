pub(super) const fn compute_u64_mask(width: u32) -> u64 {
    if width == 0 {
        return 0;
    }
    if width == u64::BITS {
        // this would fail in checked shl,
        // but the mask is just full of ones
        return 0u64.wrapping_sub(1u64);
    }
    let num_values = u64::checked_shl(1u64, width);
    if let Some(num_values) = num_values {
        num_values.wrapping_sub(1u64)
    } else {
        panic!("Bit mask length should fit");
    }
}

pub(super) const fn compute_u64_sign_bit_mask(width: u32) -> u64 {
    if width == 0 {
        return 0;
    }
    // the highest bit within mask (unless length is 0)
    let result = 1u64.checked_shl(width - 1);
    if let Some(result) = result {
        result
    } else {
        panic!("Sign bit mask length should fit")
    }
}

pub(super) fn is_u64_highest_bit_set(value: u64, width: u32) -> bool {
    value & compute_u64_sign_bit_mask(width) != 0
}
