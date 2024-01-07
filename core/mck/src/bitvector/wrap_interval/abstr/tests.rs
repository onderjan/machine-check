use crate::bitvector::concrete::ConcreteBitvector;

use super::Bitvector;

mod op;

#[test]
fn support() {
    let a = ConcreteBitvector::<16>::new(0x1337);
    let b = ConcreteBitvector::<16>::new(0xCAFE);

    let umin = ConcreteBitvector::<16>::new(0);
    let umax = ConcreteBitvector::<16>::new(0xFFFF);
    let smin = ConcreteBitvector::<16>::new(0x8000);
    let smax = ConcreteBitvector::<16>::new(0x7FFF);

    let a_b = Bitvector::from_wrap_interval(a, b);
    assert!(a_b.contains(&a_b));
    assert!(a_b.contains_concrete(&a));
    assert!(a_b.contains_concrete(&b));
    assert!(!a_b.contains_concrete(&umin));
    assert!(!a_b.contains_concrete(&umax));
    assert!(a_b.contains_concrete(&smin));
    assert!(a_b.contains_concrete(&smax));

    let b_a = Bitvector::from_wrap_interval(b, a);
    assert!(b_a.contains(&b_a));
    assert!(b_a.contains_concrete(&a));
    assert!(b_a.contains_concrete(&b));
    assert!(b_a.contains_concrete(&umin));
    assert!(b_a.contains_concrete(&umax));
    assert!(!b_a.contains_concrete(&smin));
    assert!(!b_a.contains_concrete(&smax));

    assert!(!a_b.contains(&b_a));
    assert!(!b_a.contains(&a_b));

    let umin_umax = Bitvector::from_wrap_interval(umin, umax);
    let umax_umin = Bitvector::from_wrap_interval(umax, umin);
    assert!(umin_umax.contains(&umax_umin));
    assert!(!umax_umin.contains(&umin_umax));

    assert!(umin_umax.contains(&a_b));
    assert!(!umax_umin.contains(&a_b));
    assert!(!a_b.contains(&umin_umax));
    assert!(!a_b.contains(&umax_umin));

    assert!(umin_umax.contains(&b_a));
    assert!(!umax_umin.contains(&b_a));
    assert!(!b_a.contains(&umin_umax));
    assert!(b_a.contains(&umax_umin));

    let smin_smax = Bitvector::from_wrap_interval(smin, smax);
    let smax_smin = Bitvector::from_wrap_interval(smax, smin);
    assert!(smin_smax.contains(&smax_smin));
    assert!(!smax_smin.contains(&smin_smax));

    assert!(smin_smax.contains(&a_b));
    assert!(!smax_smin.contains(&a_b));
    assert!(!a_b.contains(&smin_smax));
    assert!(a_b.contains(&smax_smin));

    assert!(smin_smax.contains(&b_a));
    assert!(!smax_smin.contains(&b_a));
    assert!(!b_a.contains(&smin_smax));
    assert!(!b_a.contains(&smax_smin));

    let umin_smax = Bitvector::from_wrap_interval(umin, smax);
    assert!(!umin_smax.contains(&a_b));
    assert!(!umin_smax.contains(&b_a));
    assert!(!a_b.contains(&umin_smax));
    assert!(!b_a.contains(&umin_smax));

    let smin_umax = Bitvector::from_wrap_interval(smin, umax);
    assert!(!smin_umax.contains(&a_b));
    assert!(!smin_umax.contains(&b_a));
    assert!(!a_b.contains(&smin_umax));
    assert!(!b_a.contains(&smin_umax));
}
