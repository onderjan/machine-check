use crate::{
    bitvector::{
        abstr::dual_interval::DualInterval,
        interval::{
            SignedInterval, SignlessInterval, UnsignedInterval, WrappingInterpretation,
            WrappingInterval,
        },
    },
    concr::ConcreteBitvector,
    misc::BitvectorBound,
};

impl<B: BitvectorBound> DualInterval<B> {
    pub(super) fn opt_halves(self) -> (Option<SignlessInterval<B>>, Option<SignlessInterval<B>>) {
        if self.near_half == self.far_half {
            if self.near_half.is_sign_bit_set() {
                (None, Some(self.far_half))
            } else {
                (Some(self.near_half), None)
            }
        } else {
            (Some(self.near_half), Some(self.far_half))
        }
    }

    pub(super) fn try_from_opt_halves(
        near_half: Option<SignlessInterval<B>>,
        far_half: Option<SignlessInterval<B>>,
    ) -> Option<Self> {
        // if one is not present, take the other
        let (near_half, far_half) = match (near_half, far_half) {
            (None, None) => return None,
            (None, Some(far_half)) => (far_half, far_half),
            (Some(near_half), None) => (near_half, near_half),
            (Some(near_half), Some(far_half)) => {
                assert_eq!(near_half.bound(), far_half.bound());
                (near_half, far_half)
            }
        };

        Some(Self {
            near_half,
            far_half,
        })
    }

    pub(super) fn from_opt_halves(
        near_half: Option<SignlessInterval<B>>,
        far_half: Option<SignlessInterval<B>>,
    ) -> Self {
        Self::try_from_opt_halves(near_half, far_half)
            .expect("At least one half should be supplied")
    }
}

pub fn wrapping_halves<B: BitvectorBound>(
    interval: WrappingInterval<B>,
) -> (Option<SignlessInterval<B>>, Option<SignlessInterval<B>>) {
    let bound = interval.bound();
    let interpreted = interval.interpret();

    match interpreted {
        WrappingInterpretation::Signless(interval) => {
            let far_half = interval.min().is_sign_bit_set();
            if far_half {
                (None, Some(interval))
            } else {
                (Some(interval), None)
            }
        }
        WrappingInterpretation::Unsigned(interval) => (
            Some(SignlessInterval::new(
                interval.min().cast_bitvector(),
                ConcreteBitvector::<B>::new_underhalf(bound),
            )),
            Some(SignlessInterval::new(
                ConcreteBitvector::<B>::new_overhalf(bound),
                interval.max().cast_bitvector(),
            )),
        ),
        WrappingInterpretation::Signed(interval) => (
            Some(SignlessInterval::new(
                ConcreteBitvector::<B>::zero(interval.bound()),
                interval.max().cast_bitvector(),
            )),
            Some(SignlessInterval::new(
                interval.min().cast_bitvector(),
                ConcreteBitvector::<B>::new_umax(bound),
            )),
        ),
    }
}

pub fn unsigned_halves<B: BitvectorBound>(
    interval: UnsignedInterval<B>,
) -> (Option<SignlessInterval<B>>, Option<SignlessInterval<B>>) {
    let bound = interval.bound();
    match interval.try_into_signless() {
        Some(interval) => {
            let far_half = interval.is_sign_bit_set();
            if far_half {
                (None, Some(interval))
            } else {
                (Some(interval), None)
            }
        }
        None => (
            Some(SignlessInterval::new(
                interval.min().cast_bitvector(),
                ConcreteBitvector::<B>::new_underhalf(bound),
            )),
            Some(SignlessInterval::new(
                ConcreteBitvector::<B>::new_overhalf(bound),
                interval.max().cast_bitvector(),
            )),
        ),
    }
}

pub fn signed_halves<B: BitvectorBound>(
    interval: SignedInterval<B>,
) -> (Option<SignlessInterval<B>>, Option<SignlessInterval<B>>) {
    let bound = interval.bound();
    match interval.try_into_signless() {
        Some(interval) => {
            let far_half = interval.is_sign_bit_set();
            if far_half {
                (None, Some(interval))
            } else {
                (Some(interval), None)
            }
        }
        None => (
            Some(SignlessInterval::new(
                ConcreteBitvector::<B>::zero(bound),
                interval.max().cast_bitvector(),
            )),
            Some(SignlessInterval::new(
                interval.min().cast_bitvector(),
                ConcreteBitvector::<B>::new_umax(bound),
            )),
        ),
    }
}
