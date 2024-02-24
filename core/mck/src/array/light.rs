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
        Self::bi_func(
            self.inner.iter().map(|e| (*e.0, e.1)),
            other.inner.iter().map(|e| (*e.0, e.1)),
            |accumulator, lhs, rhs| ControlFlow::Continue(func(accumulator, lhs, rhs)),
            init,
        )
    }

    pub fn subsume(&mut self, other: Self, func: fn(&mut T, T)) {
        Self::bi_func(
            self.inner.iter_mut().map(|e| (*e.0, e.1)),
            other.inner.into_iter(),
            |_, lhs, rhs| {
                (func)(*lhs, rhs.clone());
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

    pub fn involve<U: Debug + Clone>(&mut self, other: &LightArray<U>, func: fn(&mut T, &U)) {
        Self::bi_func(
            self.inner.iter_mut().map(|e| (*e.0, e.1)),
            other.inner.iter().map(|e| (*e.0, e.1)),
            |_, lhs, rhs| {
                (func)(*lhs, *rhs);
                ControlFlow::Continue(())
            },
            (),
        );
    }

    pub fn involve_with_flow<U: Debug + Clone, R>(
        &mut self,
        other: &LightArray<U>,
        func: fn(R, &mut T, &U) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        Self::bi_func(
            self.inner.iter_mut().map(|e| (*e.0, e.1)),
            other.inner.iter().map(|e| (*e.0, e.1)),
            |result, lhs, rhs| func(result, *lhs, *rhs),
            default_result,
        )
    }

    fn bi_func<U, V, R>(
        lhs_iter: impl Iterator<Item = (usize, U)>,
        rhs_iter: impl Iterator<Item = (usize, V)>,
        func: impl Fn(R, &mut U, &mut V) -> ControlFlow<R, R>,
        default_result: R,
    ) -> R {
        let mut lhs_iter = lhs_iter.peekable();
        let mut rhs_iter = rhs_iter.peekable();

        let mut lhs_current = lhs_iter
            .next()
            .expect("Expected at least one light map entry");
        let mut rhs_current = rhs_iter
            .next()
            .expect("Expected at least one light map entry");

        let mut result = default_result;
        let result = loop {
            match (func)(result, &mut lhs_current.1, &mut rhs_current.1) {
                ControlFlow::Continue(next_result) => {
                    // continue normally
                    result = next_result;
                }
                ControlFlow::Break(next_result) => break next_result,
            }

            // move to the next index
            let lhs_next_index = lhs_iter.peek().map(|e| e.0);
            let rhs_next_index = rhs_iter.peek().map(|e| e.0);
            match (lhs_next_index, rhs_next_index) {
                (None, None) => break result,
                (None, Some(_)) => rhs_current = rhs_iter.next().unwrap(),
                (Some(_), None) => lhs_current = lhs_iter.next().unwrap(),
                (Some(lhs_next_index), Some(rhs_next_index)) => {
                    match lhs_next_index.cmp(&rhs_next_index) {
                        std::cmp::Ordering::Less => {
                            // next lhs index is smaller, move it
                            lhs_current = lhs_iter.next().unwrap();
                        }
                        std::cmp::Ordering::Equal => {
                            // both next indices are equal, move both
                            lhs_current = lhs_iter.next().unwrap();
                            rhs_current = rhs_iter.next().unwrap();
                        }
                        std::cmp::Ordering::Greater => {
                            // next rhs index is smaller, move it
                            rhs_current = rhs_iter.next().unwrap();
                        }
                    }
                }
            }
        };
        result
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
