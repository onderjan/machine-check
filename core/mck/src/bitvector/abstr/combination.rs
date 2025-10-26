use std::{hash::Hash, marker::PhantomData};

use crate::{
    abstr::{
        dual_interval::DualInterval, eq_domain::EqualityDomain, three_valued::ThreeValuedBitvector,
        BitvectorDomain,
    },
    bitvector::interval::WrappingInterval,
    misc::BitvectorBound,
};

pub trait DomainCombination<B: BitvectorBound>: Clone + Copy + Hash {
    type Left: BitvectorDomain<Bound = B>;
    type Right: BitvectorDomain<Bound = B>;
    type General<X: BitvectorBound>: DomainCombination<
        X,
        Left = <Self::Left as BitvectorDomain>::General<X>,
        Right = <Self::Right as BitvectorDomain>::General<X>,
    >;

    fn combine(left: &mut Self::Left, right: &mut Self::Right);
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct TVDICombination<B: BitvectorBound>(PhantomData<B>);

impl<B: BitvectorBound> DomainCombination<B> for TVDICombination<B> {
    type Left = ThreeValuedBitvector<B>;
    type Right = DualInterval<B>;

    type General<X: BitvectorBound> = TVDICombination<X>;

    fn combine(three_valued: &mut Self::Left, dual_interval: &mut Self::Right) {
        // restrict the dual interval
        let near_min = three_valued.umin().max(dual_interval.umin());
        let near_max = three_valued.smax().min(dual_interval.smax());
        let far_min = three_valued.smin().max(dual_interval.smin());
        let far_max = three_valued.umax().min(dual_interval.umax());

        let near = WrappingInterval::new(near_min.cast_bitvector(), near_max.cast_bitvector());
        let far = WrappingInterval::new(far_min.cast_bitvector(), far_max.cast_bitvector());

        *dual_interval = DualInterval::from_wrapping_intervals(&[near, far]);

        // restrict the three-valued bit-vector
        let interval_bitvec = ThreeValuedBitvector::from_unsigned_interval(near_min, far_max);
        let Some(three_valued_result) = three_valued.meet(&interval_bitvec) else {
            panic!("Three-valued bit-vector combined with dual-interval should not be empty");
        };

        *three_valued = three_valued_result;
    }
}

#[derive(Clone, Copy, Hash, Debug)]
pub struct TVEQCombination<B: BitvectorBound>(PhantomData<B>);

impl<B: BitvectorBound> DomainCombination<B> for TVEQCombination<B> {
    type Left = ThreeValuedBitvector<B>;
    type Right = EqualityDomain<B>;

    type General<X: BitvectorBound> = TVEQCombination<X>;

    fn combine(three_valued: &mut Self::Left, eq_domain: &mut Self::Right) {
        // restrictions only happen from constant values

        match (three_valued.concrete_value(), eq_domain.concrete_value()) {
            (None, None) => {
                // do nothing
            }
            (None, Some(eq_domain_concrete)) => {
                // restrict the three-valued domain
                assert!(three_valued.contains_concrete(&eq_domain_concrete));
                *three_valued = Self::Left::single_value(eq_domain_concrete);
            }
            (Some(three_valued_concrete), None) => {
                // restrict the equality domain
                eq_domain.force_concrete(three_valued_concrete);
            }
            (Some(three_valued_concrete), Some(eq_domain_concrete)) => {
                // they must be equal
                assert_eq!(three_valued_concrete, eq_domain_concrete);
            }
        }
    }
}
