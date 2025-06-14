use core::panic;

use machine_check_common::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO};

use crate::{
    concr::{PanicResult, Test},
    forward::{Bitwise, Ext, HwArith, HwShift, TypedCmp, TypedEq},
};

use super::ConcreteBitvector;

#[test]
fn support() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);

    let zero = ConcreteBitvector::<16>::new(0);
    let full = ConcreteBitvector::<16>::new(0xFFFF);
    let min = ConcreteBitvector::<16>::new(0x8000);

    assert_eq!(a.to_u64(), 0xCAFE);
    assert_eq!(b.to_u64(), 0x1337);
    assert_eq!(zero.to_u64(), 0);
    assert_eq!(full.to_u64(), 0xFFFF);
    assert_eq!(min.to_u64(), 0x8000);

    assert_eq!(a.to_i64(), -0x3502);
    assert_eq!(b.to_i64(), 0x1337);
    assert_eq!(zero.to_i64(), 0);
    assert_eq!(full.to_i64(), -1);
    assert_eq!(min.to_i64(), -0x8000);

    assert!(a.is_nonzero());
    assert!(b.is_nonzero());
    assert!(!zero.is_nonzero());
    assert!(full.is_nonzero());
    assert!(min.is_nonzero());

    assert!(!a.is_zero());
    assert!(!b.is_zero());
    assert!(zero.is_zero());
    assert!(!full.is_zero());
    assert!(!min.is_zero());

    assert!(a.is_sign_bit_set());
    assert!(!b.is_sign_bit_set());
    assert!(!zero.is_sign_bit_set());
    assert!(full.is_sign_bit_set());
    assert!(min.is_sign_bit_set());

    assert_eq!(min, ConcreteBitvector::sign_bit_mask());
    assert_eq!(full, ConcreteBitvector::bit_mask());

    assert_eq!(
        ConcreteBitvector::<8>::all_with_length_iter().count(),
        2usize.pow(8)
    );
}

#[test]
fn eq() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);

    assert!(a.eq(a).into_bool());
    assert!(b.eq(b).into_bool());
    assert!(!a.eq(b).into_bool());
    assert!(!b.eq(a).into_bool());
}

#[test]
fn cmp() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);

    // identity
    assert!(!a.ult(a).into_bool());
    assert!(a.ule(a).into_bool());
    assert!(!a.slt(a).into_bool());
    assert!(a.sle(a).into_bool());

    // comparison
    assert!(!a.ult(b).into_bool());
    assert!(!a.ule(b).into_bool());
    assert!(a.slt(b).into_bool());
    assert!(a.sle(b).into_bool());

    // try flipped the other way, they are not equal
    assert!(b.ult(a).into_bool());
    assert!(b.ule(a).into_bool());
    assert!(!b.slt(a).into_bool());
    assert!(!b.sle(a).into_bool());
}

#[test]
fn bitwise() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);

    // compare results to calculated values
    assert_eq!(a.bit_not().to_u64(), 0x3501);
    assert_eq!(b.bit_not().to_u64(), 0xECC8);

    assert_eq!(a.bit_and(b).to_u64(), 0x0236);
    assert_eq!(a.bit_or(b).to_u64(), 0xDBFF);
    assert_eq!(a.bit_xor(b).to_u64(), 0xD9C9);
}

#[test]
fn ext() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);

    // longer uext will preserve unsigned value
    assert_eq!(Ext::<32>::uext(a).to_u64(), 0xCAFE);
    assert_eq!(Ext::<32>::uext(a).to_i64(), 0xCAFE);
    assert_eq!(Ext::<32>::uext(b).to_u64(), 0x1337);
    assert_eq!(Ext::<32>::uext(b).to_i64(), 0x1337);

    // longer sext will preserve signed value
    assert_eq!(Ext::<32>::sext(a).to_u64(), 0xFFFFCAFE);
    assert_eq!(Ext::<32>::sext(a).to_i64(), -0x3502);
    assert_eq!(Ext::<32>::sext(b).to_u64(), 0x1337);
    assert_eq!(Ext::<32>::sext(b).to_i64(), 0x1337);

    // shorter ext will always just cut
    assert_eq!(Ext::<4>::uext(a).to_u64(), 0xE);
    assert_eq!(Ext::<4>::uext(a).to_i64(), -0x2);
    assert_eq!(Ext::<4>::sext(a).to_u64(), 0xE);
    assert_eq!(Ext::<4>::sext(a).to_i64(), -0x2);

    // same ext will preserve value
    assert_eq!(a, Ext::<16>::uext(a));
    assert_eq!(b, Ext::<16>::uext(b));
    assert_eq!(a, Ext::<16>::sext(a));
    assert_eq!(b, Ext::<16>::sext(b));
}

#[test]
fn shift() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);
    let four = ConcreteBitvector::<16>::new(0x4);
    let sixteen = ConcreteBitvector::<16>::new(0x16);
    let too_much = ConcreteBitvector::<16>::new(0x42);

    // shift by a reasonable value
    assert_eq!(a.logic_shl(four).to_u64(), 0xAFE0);
    assert_eq!(b.logic_shl(four).to_u64(), 0x3370);
    assert_eq!(a.logic_shr(four).to_u64(), 0x0CAF);
    assert_eq!(b.logic_shr(four).to_u64(), 0x0133);
    assert_eq!(a.arith_shr(four).to_u64(), 0xFCAF);
    assert_eq!(b.arith_shr(four).to_u64(), 0x0133);

    // shift by exactly all bits
    // should be zero except for arith shift right of negative
    assert_eq!(a.logic_shl(sixteen).to_u64(), 0x0000);
    assert_eq!(b.logic_shl(sixteen).to_u64(), 0x0000);
    assert_eq!(a.logic_shr(sixteen).to_u64(), 0x0000);
    assert_eq!(b.logic_shr(sixteen).to_u64(), 0x0000);
    assert_eq!(a.arith_shr(sixteen).to_u64(), 0xFFFF);
    assert_eq!(b.arith_shr(sixteen).to_u64(), 0x0000);

    // shift by an unreasonable value
    // should give the same results as by all bits
    assert_eq!(a.logic_shl(too_much).to_u64(), 0x0000);
    assert_eq!(b.logic_shl(too_much).to_u64(), 0x0000);
    assert_eq!(a.logic_shr(too_much).to_u64(), 0x0000);
    assert_eq!(b.logic_shr(too_much).to_u64(), 0x0000);
    assert_eq!(a.arith_shr(too_much).to_u64(), 0xFFFF);
    assert_eq!(b.arith_shr(too_much).to_u64(), 0x0000);
}

#[test]
fn arith() {
    let a = ConcreteBitvector::<16>::new(0xCAFE);
    let b = ConcreteBitvector::<16>::new(0x1337);
    let c = ConcreteBitvector::<16>::new(0x4000);
    let d = ConcreteBitvector::<16>::new(0xBADA);

    let zero = ConcreteBitvector::<16>::new(0);

    // add, sub, mul do not have corner cases except wrapping
    // compare results to calculated values
    assert_eq!(a.add(b).to_u64(), 0xDE35);
    assert_eq!(a.add(c).to_u64(), 0x0AFE);
    assert_eq!(b.add(c).to_u64(), 0x5337);
    assert_eq!(a.sub(b).to_u64(), 0xB7C7);
    assert_eq!(a.sub(c).to_u64(), 0x8AFE);
    assert_eq!(b.sub(c).to_u64(), 0xD337);
    assert_eq!(a.mul(b).to_u64(), 0x7692);
    assert_eq!(a.mul(c).to_u64(), 0x8000);
    assert_eq!(b.mul(c).to_u64(), 0xC000);

    // unsigned division and remainder have division by zero
    // try out normal first
    assert_eq!(expect_no_panic(a.udiv(b)).to_u64(), 0x000A);
    assert_eq!(expect_no_panic(a.urem(b)).to_u64(), 0x0AD8);
    assert_eq!(expect_no_panic(a.udiv(c)).to_u64(), 0x0003);
    assert_eq!(expect_no_panic(a.urem(c)).to_u64(), 0x0AFE);
    assert_eq!(expect_no_panic(b.udiv(c)).to_u64(), 0x0000);
    assert_eq!(expect_no_panic(b.urem(c)).to_u64(), 0x1337);

    // in case of unsigned division-by-zero
    // division result is all-ones and remainder is the dividend
    assert_eq!(expect_div_panic(a.udiv(zero)).to_u64(), 0xFFFF);
    assert_eq!(expect_rem_panic(a.urem(zero)).to_u64(), 0xCAFE);
    assert_eq!(expect_div_panic(b.udiv(zero)).to_u64(), 0xFFFF);
    assert_eq!(expect_rem_panic(b.urem(zero)).to_u64(), 0x1337);
    assert_eq!(expect_div_panic(c.udiv(zero)).to_u64(), 0xFFFF);
    assert_eq!(expect_rem_panic(c.urem(zero)).to_u64(), 0x4000);

    // signed division and remainder have four-quadrant behaviour,
    // division by zero and overflow
    assert_eq!(expect_no_panic(c.sdiv(b)).to_u64(), 0x0003); // positive / positive
    assert_eq!(expect_no_panic(c.srem(b)).to_u64(), 0x065B);
    assert_eq!(expect_no_panic(a.sdiv(b)).to_u64(), 0xFFFE); // negative / positive
    assert_eq!(expect_no_panic(a.srem(b)).to_u64(), 0xF16C);
    assert_eq!(expect_no_panic(c.sdiv(a)).to_u64(), 0xFFFF); // positive / negative
    assert_eq!(expect_no_panic(c.srem(a)).to_u64(), 0x0AFE);
    assert_eq!(expect_no_panic(d.sdiv(a)).to_u64(), 0x0001); // negative / negative
    assert_eq!(expect_no_panic(d.srem(a)).to_u64(), 0xEFDC);

    // in case of signed division-by-zero
    // division result is all-ones and remainder is the dividend
    // (same as unsigned)
    assert_eq!(expect_div_panic(a.sdiv(zero)).to_u64(), 0xFFFF);
    assert_eq!(expect_rem_panic(a.srem(zero)).to_u64(), 0xCAFE);
    assert_eq!(expect_div_panic(b.sdiv(zero)).to_u64(), 0xFFFF);
    assert_eq!(expect_rem_panic(b.srem(zero)).to_u64(), 0x1337);
    assert_eq!(expect_div_panic(c.sdiv(zero)).to_u64(), 0xFFFF);
    assert_eq!(expect_rem_panic(c.srem(zero)).to_u64(), 0x4000);

    // overflow only happens if the minimum value is divided by minus one
    // because the minimum value is not representable in positive
    // in that case, we expect the minimum value remain in divisor
    // and no remainder
    let min = ConcreteBitvector::<16>::new(0x8000);
    let minus_one = ConcreteBitvector::<16>::new(0xFFFF);
    assert_eq!(expect_no_panic(min.sdiv(minus_one)), min);
    assert_eq!(expect_no_panic(min.srem(minus_one)), zero);
}

fn expect_no_panic<const L: u32>(
    panic_result: PanicResult<ConcreteBitvector<L>>,
) -> ConcreteBitvector<L> {
    if panic_result.panic != ConcreteBitvector::new(PANIC_NUM_NO_PANIC) {
        panic!("Expected no panic")
    }
    panic_result.result
}

fn expect_div_panic<const L: u32>(
    panic_result: PanicResult<ConcreteBitvector<L>>,
) -> ConcreteBitvector<L> {
    if panic_result.panic != ConcreteBitvector::new(PANIC_NUM_DIV_BY_ZERO) {
        panic!("Expected division panic")
    }
    panic_result.result
}

fn expect_rem_panic<const L: u32>(
    panic_result: PanicResult<ConcreteBitvector<L>>,
) -> ConcreteBitvector<L> {
    if panic_result.panic != ConcreteBitvector::new(PANIC_NUM_REM_BY_ZERO) {
        panic!("Expected remainder panic")
    }
    panic_result.result
}
