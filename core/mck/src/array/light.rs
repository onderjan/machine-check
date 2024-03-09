use std::{
    borrow::BorrowMut,
    collections::BTreeMap,
    fmt::Debug,
    ops::{ControlFlow, Index, IndexMut},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LightArray<T: Debug + Clone> {
    len: usize,
    inner: BTreeMap<usize, T>,
}

impl<T: Debug + Clone> LightArray<T> {
    pub fn new_filled(element: T, len: usize) -> Self {
        let inner = BTreeMap::from_iter([(0, element)]);
        Self { len, inner }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn lattice_fold<B>(&self, init: B, func: fn(B, &T) -> B) -> B {
        let mut accumulator = init;
        for value in self.inner.values() {
            accumulator = (func)(accumulator, value);
        }
        accumulator
    }

    pub fn lattice_bi_fold<B: Copy>(&self, other: &Self, init: B, func: fn(B, &T, &T) -> B) -> B {
        Self::immutable_bi_func(
            self.inner.iter().map(|e| (*e.0, e.1)),
            other.inner.iter().map(|e| (*e.0, e.1)),
            |accumulator, lhs, rhs| ControlFlow::Continue(func(accumulator, lhs, rhs)),
            init,
        )
    }

    pub fn subsume(&mut self, other: Self, func: fn(&mut T, T)) {
        Self::mutable_bi_func(
            self,
            other.inner.into_iter(),
            |_, lhs, rhs| {
                (func)(lhs, rhs);
                ControlFlow::Continue(())
            },
            (),
        );
    }

    pub fn map<U: Debug + Clone>(&self, func: fn(&T) -> U) -> LightArray<U> {
        let mut result_inner = BTreeMap::new();

        for entry in self.inner.iter() {
            result_inner.insert(*entry.0, (func)(entry.1));
        }
        LightArray {
            len: self.len,
            inner: result_inner,
        }
    }

    pub fn involve<V: Debug + Clone>(&mut self, other: &LightArray<V>, func: fn(&mut T, &V)) {
        self.involve_with_flow(
            other,
            |_, lhs, rhs| {
                (func)(lhs, rhs);
                ControlFlow::Continue(())
            },
            (),
        );
    }

    pub fn involve_with_flow<V: Debug + Clone, R>(
        &mut self,
        other: &LightArray<V>,
        func: impl Fn(R, &mut T, &V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        Self::mutable_bi_func(
            self,
            other.inner.iter().map(|e| (*e.0, e.1)),
            |result, lhs, rhs| func(result, lhs, rhs),
            default_result,
        )
    }

    fn mutable_bi_func<U: Debug + Clone, V: Clone, R>(
        lhs: &mut LightArray<U>,
        rhs_iter: impl Iterator<Item = (usize, V)>,
        func: impl Fn(R, &mut U, V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        let mut rhs_iter = rhs_iter.peekable();
        let (mut index, mut rhs_current) = rhs_iter
            .next()
            .expect("Expected at least one light map entry");
        assert_eq!(index, 0);

        let mut lhs_previous = lhs
            .inner
            .get(&index)
            .expect("Expected light map entry at index 0")
            .clone();

        let mut result = default_result;
        let mut next_break = false;
        loop {
            use std::ops::Bound::{Excluded, Unbounded};

            // if there is no current lhs at the index, insert it from previous
            let lhs_current = if let Some(lhs_current) = lhs.inner.get_mut(&index) {
                lhs_current
            } else {
                lhs.inner.insert(index, lhs_previous);
                lhs.inner.get_mut(&index).unwrap()
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
            let lhs_next_index = lhs
                .inner
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
        lhs_iter: impl Iterator<Item = (usize, &'a U)>,
        rhs_iter: impl Iterator<Item = (usize, &'a V)>,
        func: impl Fn(R, &U, &V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        let mut lhs_iter = lhs_iter.peekable();
        let mut rhs_iter = rhs_iter.peekable();

        let (lhs_index, mut lhs_current) = lhs_iter
            .next()
            .expect("Expected at least one light map entry");
        assert_eq!(lhs_index, 0);
        let (rhs_index, mut rhs_current) = rhs_iter
            .next()
            .expect("Expected at least one light map entry");
        assert_eq!(rhs_index, 0);

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
}

impl<T: Debug + Clone> Debug for LightArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for ((current_index, current_element), next_index) in self.inner.iter().zip(
            self.inner
                .iter()
                .skip(1)
                .map(|v| Some(v.0))
                .chain(std::iter::once(None)),
        ) {
            let next_index = if let Some(next_index) = next_index {
                *next_index
            } else {
                self.len
            };
            if next_index != *current_index + 1 {
                write!(
                    f,
                    "{}..={}: {:?}, ",
                    current_index,
                    next_index - 1,
                    current_element
                )?;
            } else {
                write!(f, "{}: {:?}, ", current_index, current_element)?;
            }
        }
        write!(f, "}}")
    }
}

impl<T: Debug + Clone> Index<usize> for LightArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);

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

impl<T: Debug + Clone> IndexMut<usize> for LightArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);

        use std::ops::Bound::{Included, Unbounded};

        // we have to insert both the value and also the next value
        // if it is within array bounds and does not exist

        let element = self
            .inner
            .range((Unbounded, Included(index)))
            .last()
            .expect("Expected lower bound entry when indexing")
            .1
            .clone();

        let next_index = index + 1;
        if next_index < self.len {
            // needed to preserve the next elements
            self.inner.entry(next_index).or_insert(element.clone());
        }

        self.inner.entry(index).or_insert(element).borrow_mut()
    }
}
