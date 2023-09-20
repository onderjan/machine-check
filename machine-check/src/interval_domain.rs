use std::{
    num::Wrapping,
    ops::{Add, Sub},
};

use machine_check_traits::{MachineCastToUnsigned, MachinePrimitive};
use num::traits::{CheckedAdd, One, WrappingSub};

pub fn w<T>(e: T) -> Wrapping<T> {
    Wrapping::<T>(e)
}

#[derive(Clone, Copy, Debug)]
pub struct IntervalDomain<T> {
    min: T,
    max: T,
}

impl<T: MachinePrimitive> IntervalDomain<Wrapping<T>> {
    #[allow(dead_code)]
    pub fn from_concrete(concrete: T) -> IntervalDomain<Wrapping<T>> {
        IntervalDomain {
            min: w(concrete),
            max: w(concrete),
        }
    }

    #[allow(dead_code)]
    pub fn from_interval(min: T, max: T) -> IntervalDomain<Wrapping<T>> {
        IntervalDomain {
            min: w(min),
            max: w(max),
        }
    }
}

impl<T> IntervalDomain<Wrapping<T>>
where
    T: MachinePrimitive,
{
    fn halfopen_size(&self) -> <T as MachineCastToUnsigned>::Unsigned {
        let unsigned_self_min = self.min.0.cast_to_unsigned();
        let unsigned_self_max = self.max.0.cast_to_unsigned();
        unsigned_self_max.wrapping_sub(&unsigned_self_min)
    }

    fn addsub_nonrepresentable(&self, other: &Self) -> Option<IntervalDomain<Wrapping<T>>> {
        // interval size is self halfopen + other halfopen + 1
        let result_halfopen_size = self.halfopen_size().checked_add(&other.halfopen_size());
        if let Some(result_halfopen_size) = result_halfopen_size {
            // halfopen interval size is representable in type, add 1
            let unsigned_one = <T as MachineCastToUnsigned>::Unsigned::one();
            let interval_size = result_halfopen_size.checked_add(&unsigned_one);
            if interval_size.is_some() {
                // interval size is representable in type, do not return anything
                return None;
            }
        };
        // interval size not representable in type, return full interval to be used
        Some(IntervalDomain {
            min: w(T::min_value()),
            max: w(T::max_value()),
        })
    }
}

impl<T> Add for IntervalDomain<Wrapping<T>>
where
    T: MachinePrimitive,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.addsub_nonrepresentable(&other)
            .unwrap_or_else(|| IntervalDomain::<Wrapping<T>> {
                min: w(T::wrapping_add(&self.min.0, &other.min.0)),
                max: w(T::wrapping_add(&self.max.0, &other.max.0)),
            })
    }
}

impl<T> Sub for IntervalDomain<Wrapping<T>>
where
    T: MachinePrimitive,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.addsub_nonrepresentable(&other)
            .unwrap_or_else(|| IntervalDomain {
                min: w(T::wrapping_sub(&self.min.0, &other.min.0)),
                max: w(T::wrapping_sub(&self.max.0, &other.max.0)),
            })
    }
}
