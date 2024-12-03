use std::borrow::BorrowMut;
use std::ops::{Add, Sub};
use std::rc::Rc;
use std::{
    collections::BTreeMap,
    fmt::Debug,
    ops::{ControlFlow, Index},
};

use num::{One, Zero};

#[cfg(test)]
mod tests;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LightArray<
    I: Clone
        + Copy
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Zero
        + One
        + Add<Output = I>
        + Sub<Output = I>,
    E: Clone + PartialEq + Eq,
> {
    inner: Rc<BTreeMap<I, E>>,
}

impl<
        I: Clone
            + Copy
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Zero
            + One
            + Add<Output = I>
            + Sub<Output = I>,
        E: Clone + PartialEq + Eq,
    > LightArray<I, E>
{
    pub fn new_filled(element: E) -> Self {
        let zero_index = <I as Zero>::zero();
        let inner = Rc::new(BTreeMap::from_iter([(zero_index, element)]));
        Self { inner }
    }

    pub fn write(&mut self, index: I, value: E) {
        use std::ops::Bound::{Included, Unbounded};

        // first, we will get the previous value
        let previous_value = self
            .inner
            .range((Unbounded, Included(index)))
            .last()
            .expect("Expected lower bound entry when reading for write")
            .1;

        // if the previous value is equal to the written value, do nothing
        if *previous_value == value {
            return;
        }

        let previous_value = previous_value.clone();

        // insert the written value
        let inner = Rc::make_mut(&mut self.inner);
        inner.insert(index, value);

        // we need to preserve the previous value of the next elements
        // insert the previous value immediately after the written one
        // if it does not already exist

        let next_index = index + <I as One>::one();
        if next_index != <I as Zero>::zero() {
            inner.entry(next_index).or_insert(previous_value);
        }

        // it is guaranteed that we have not created a situation with successive two map entries
        // that have the same values
    }

    fn indexed_iter(&self, min_index: I, max_index: Option<I>) -> impl Iterator<Item = &E> {
        // the lower element may not be within the range, find it specifically
        use std::ops::Bound::{Excluded, Included, Unbounded};
        let lower_element = self
            .inner
            .range((Unbounded, Included(min_index)))
            .last()
            .expect("Expected lower bound entry when iterator-indexing")
            .1;

        let max_bound = if let Some(max_index) = max_index {
            assert!(min_index <= max_index);
            Included(max_index)
        } else {
            Unbounded
        };
        let other_elements_iter = self
            .inner
            .range((Excluded(min_index), max_bound))
            .map(|(_index, value)| value);
        std::iter::once(lower_element).chain(other_elements_iter)
    }

    pub fn fold_indexed<B>(
        &self,
        min_index: I,
        max_index: Option<I>,
        init: B,
        func: fn(B, &E) -> B,
    ) -> B {
        let mut accumulator = init;
        for value in self.indexed_iter(min_index, max_index) {
            accumulator = (func)(accumulator, value);
        }
        accumulator
    }

    pub fn fold<B>(&self, init: B, func: fn(B, &E) -> B) -> B {
        self.fold_indexed(<I as Zero>::zero(), None, init, func)
    }

    pub fn reduce_indexed(&self, min_index: I, max_index: Option<I>, func: fn(E, &E) -> E) -> E {
        let mut result = self
            .indexed_iter(min_index, max_index)
            .next()
            .cloned()
            .expect("Indexed iterator should have at least one element");
        for value in self.indexed_iter(min_index, max_index) {
            result = (func)(result, value);
        }
        result
    }

    pub fn map_inplace_indexed(
        &mut self,
        min_index: I,
        max_index: Option<I>,
        map_fn: impl Fn(E) -> E,
    ) {
        // the lower element may not be within the range, find it specifically
        use std::ops::Bound::{Excluded, Included, Unbounded};

        let inner = Rc::make_mut(&mut self.inner);

        let below_low_value = inner
            .range((Unbounded, Excluded(min_index)))
            .last()
            .map(|(_, value)| value.clone());

        // insert the minimum index if not already present, will be pruned after mapping if necessary
        if let Some(ref below_low_value) = below_low_value {
            inner.entry(min_index).or_insert(below_low_value.clone());
        }

        // also find the previous value of the high element in this range
        // to be used when deciding if a backstop should be added after this range
        let max_bound = if let Some(max_index) = max_index {
            assert!(min_index <= max_index);
            Included(max_index)
        } else {
            Unbounded
        };

        let old_high_value = inner
            .range((Unbounded, max_bound))
            .last()
            .expect("Expected upper bound entry when mapping")
            .1
            .clone();

        let range_vec: Vec<I> = inner
            .range((Included(min_index), max_bound))
            .map(|a| *a.0)
            .collect();

        // find the previous value before the first element, there will not be any if the min index is 0
        let mut previous_value = below_low_value;

        for index in range_vec {
            // map the value
            let value = inner.get_mut(&index).expect("The index should be in map");
            *value = map_fn(value.clone());
            let current_value = value.clone();
            // remove the entry if its value is the same as the previous value
            if let Some(previous_value) = previous_value {
                if current_value == previous_value {
                    inner.remove(&index);
                }
            }
            // update the previous value
            previous_value = Some(current_value);
        }

        let index_one = <I as One>::one();

        // if the old high value is not the same as the previous value,
        // insert it if necessary
        if let (Some(previous_value), Some(max_index)) = (previous_value, max_index) {
            if old_high_value != previous_value {
                let above_max_index = max_index + index_one;
                if above_max_index != <I as Zero>::zero() {
                    inner.entry(above_max_index).or_insert(old_high_value);
                }
            }
        }
    }

    pub fn bi_fold<B: Copy>(&self, other: &Self, init: B, func: fn(B, &E, &E) -> B) -> B {
        Self::immutable_bi_func(
            self.inner.iter().map(|e| (*e.0, e.1)),
            other.inner.iter().map(|e| (*e.0, e.1)),
            |accumulator, lhs, rhs| ControlFlow::Continue(func(accumulator, lhs, rhs)),
            init,
        )
    }

    pub fn subsume(&mut self, other: Self, func: fn(&mut E, E)) {
        // unwrap or clone other so it can be mutated
        let other_inner = Rc::try_unwrap(other.inner).unwrap_or_else(|rc| (*rc).clone());

        Self::mutable_bi_func(
            self,
            other_inner.into_iter(),
            |_, lhs, rhs| {
                (func)(lhs, rhs);
                ControlFlow::Continue(())
            },
            (),
        );
    }

    pub fn map<U: Debug + Clone + PartialEq + Eq>(&self, func: fn(&E) -> U) -> LightArray<I, U> {
        let mut result_inner = BTreeMap::new();

        for entry in self.inner.iter() {
            result_inner.insert(*entry.0, (func)(entry.1));
        }
        LightArray {
            inner: Rc::new(result_inner),
        }
    }

    pub fn involve<V: Debug + Clone + PartialEq + Eq>(
        &mut self,
        other: &LightArray<I, V>,
        func: fn(&mut E, &V),
    ) {
        self.involve_with_flow(
            other,
            |_, lhs, rhs| {
                (func)(lhs, rhs);
                ControlFlow::Continue(())
            },
            (),
        );
    }

    pub fn involve_with_flow<V: Debug + Clone + PartialEq + Eq, R>(
        &mut self,
        other: &LightArray<I, V>,
        func: impl Fn(R, &mut E, &V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        Self::mutable_bi_func(
            self,
            other.inner.iter().map(|e| (*e.0, e.1)),
            |result, lhs, rhs| func(result, lhs, rhs),
            default_result,
        )
    }

    fn mutable_bi_func<U: Clone + PartialEq + Eq, V: Clone, R>(
        lhs: &mut LightArray<I, U>,
        rhs_iter: impl Iterator<Item = (I, V)>,
        func: impl Fn(R, &mut U, V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        let lhs_inner = Rc::make_mut(&mut lhs.inner);

        let mut rhs_iter = rhs_iter.peekable();
        let (mut index, mut rhs_current) = rhs_iter
            .next()
            .expect("Expected at least one light map entry");
        assert!(index == <I as Zero>::zero());

        let mut lhs_previous = lhs_inner
            .get(&index)
            .expect("Expected light map entry at index 0")
            .clone();

        let mut result = default_result;
        let mut next_break = false;
        loop {
            use std::ops::Bound::{Excluded, Unbounded};

            // if there is no current lhs at the index, insert it from previous
            let lhs_current = if let Some(lhs_current) = lhs_inner.get_mut(&index) {
                lhs_current
            } else {
                lhs_inner.insert(index, lhs_previous);
                lhs_inner.get_mut(&index).unwrap()
            };
            lhs_previous = lhs_current.clone();

            if next_break {
                break result;
            }

            match (func)(result, lhs_current, rhs_current.clone()) {
                ControlFlow::Continue(next_result) => {
                    // continue normally
                    result = next_result;
                }
                ControlFlow::Break(next_result) => {
                    // break only after updating the next element
                    result = next_result;
                    next_break = true;
                }
            }

            // move to the next index
            let lhs_next_index = lhs_inner
                .range_mut((Excluded(index), Unbounded))
                .next()
                .map(|a| *a.0);
            let rhs_next_index = rhs_iter.peek().map(|a| a.0);

            let (next_index, move_rhs) = match (lhs_next_index, rhs_next_index) {
                (None, None) => break result,
                (Some(next_index), None) => (next_index, false),
                (None, Some(next_index)) => (next_index, true),
                (Some(lhs_next_index), Some(rhs_next_index)) => {
                    match lhs_next_index.cmp(&rhs_next_index) {
                        std::cmp::Ordering::Less => {
                            // next lhs index is smaller
                            (lhs_next_index, false)
                        }
                        std::cmp::Ordering::Equal => {
                            // both next indices are equal, move rhs
                            (rhs_next_index, true)
                        }
                        std::cmp::Ordering::Greater => {
                            // next rhs index is smaller, move rhs
                            (rhs_next_index, true)
                        }
                    }
                }
            };

            if move_rhs {
                rhs_current = rhs_iter.next().unwrap().1;
            }

            index = next_index;
        }
    }

    fn immutable_bi_func<'a, U: 'a, V: 'a, R>(
        lhs_iter: impl Iterator<Item = (I, &'a U)>,
        rhs_iter: impl Iterator<Item = (I, &'a V)>,
        func: impl Fn(R, &U, &V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        let mut lhs_iter = lhs_iter.peekable();
        let mut rhs_iter = rhs_iter.peekable();

        let (lhs_index, mut lhs_current) = lhs_iter
            .next()
            .expect("Expected at least one light map entry");
        assert!(lhs_index == <I as Zero>::zero());
        let (rhs_index, mut rhs_current) = rhs_iter
            .next()
            .expect("Expected at least one light map entry");
        assert!(rhs_index == <I as Zero>::zero());

        let mut result = default_result;
        loop {
            match (func)(result, lhs_current, rhs_current) {
                ControlFlow::Continue(next_result) => {
                    // continue normally
                    result = next_result;
                }
                ControlFlow::Break(next_result) => break next_result,
            }

            // move to the next index
            let lhs_next_index = lhs_iter.peek().map(|e| e.0);
            let rhs_next_index = rhs_iter.peek().map(|e| e.0);

            let (move_lhs, move_rhs) = match (lhs_next_index, rhs_next_index) {
                (None, None) => break result,
                (Some(_), None) => (true, false),
                (None, Some(_)) => (false, true),
                (Some(lhs_next_index), Some(rhs_next_index)) => {
                    match lhs_next_index.cmp(&rhs_next_index) {
                        std::cmp::Ordering::Less => {
                            // next lhs index is smaller, move it
                            (true, false)
                        }
                        std::cmp::Ordering::Equal => {
                            // both next indices are equal, move both
                            (true, true)
                        }
                        std::cmp::Ordering::Greater => {
                            // next rhs index is smaller, move it
                            (false, true)
                        }
                    }
                }
            };
            if move_lhs {
                lhs_current = lhs_iter.next().unwrap().1;
            }
            if move_rhs {
                rhs_current = rhs_iter.next().unwrap().1;
            }
        }
    }

    pub fn mutable_index(&mut self, index: I) -> &mut E {
        // currently retained but not available by IndexMut
        // TODO: remove as it does not keep representation compact

        use std::ops::Bound::{Included, Unbounded};

        let inner = Rc::make_mut(&mut self.inner);

        // we have to insert both the value and also the next value
        // if it is within array bounds and does not exist

        let element = inner
            .range((Unbounded, Included(index)))
            .last()
            .expect("Expected lower bound entry when indexing")
            .1
            .clone();

        let next_index = index + <I as One>::one();
        if next_index != <I as Zero>::zero() {
            // needed to preserve the next elements
            inner.entry(next_index).or_insert(element.clone());
        }

        inner.entry(index).or_insert(element).borrow_mut()
    }

    pub fn light_iter(&self) -> impl Iterator<Item = (&I, &E)> {
        self.inner.iter()
    }
}

impl<
        I: Debug
            + Clone
            + Copy
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Zero
            + One
            + Add<Output = I>
            + Sub<Output = I>,
        E: Debug + Clone + PartialEq + Eq,
    > Debug for LightArray<I, E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for ((current_index, current_element), next_index) in self.inner.iter().zip(
            self.inner
                .iter()
                .skip(1)
                .map(|v| Some(v.0))
                .chain(std::iter::once(None)),
        ) {
            let next_index_minus_one = if let Some(next_index) = next_index {
                *next_index - <I as One>::one()
            } else {
                <I as Zero>::zero() - <I as One>::one()
            };
            if next_index_minus_one != *current_index {
                write!(
                    f,
                    "{:?}..={:?}: {:?}, ",
                    current_index, next_index_minus_one, current_element
                )?;
            } else {
                write!(f, "{:?}: {:?}, ", current_index, current_element)?;
            }
        }
        write!(f, "}}")
    }
}

impl<
        I: Clone
            + Copy
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Zero
            + One
            + Add<Output = I>
            + Sub<Output = I>,
        E: Clone + PartialEq + Eq,
    > Index<I> for LightArray<I, E>
{
    type Output = E;

    fn index(&self, index: I) -> &Self::Output {
        use std::ops::Bound::{Included, Unbounded};

        // we can return the lower bound

        let lower_bound_entry = self
            .inner
            .range((Unbounded, Included(index)))
            .last()
            .expect("Expected lower bound entry when indexing");

        lower_bound_entry.1
    }
}
