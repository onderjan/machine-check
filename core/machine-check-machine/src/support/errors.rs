use std::error::Error;

use vec1::Vec1;

pub struct Errors<E: Error> {
    errors: Vec1<E>,
}

impl<E: Error> Errors<E> {
    pub fn single(error: E) -> Self {
        Errors {
            errors: Vec1::new(error),
        }
    }

    pub fn iter_to_result(iter: impl IntoIterator<Item = E>) -> Result<(), Self> {
        let mut iter = iter.into_iter();
        let Some(first_error) = iter.next() else {
            // no errors
            return Ok(());
        };
        let mut errors = Vec1::new(first_error);
        errors.extend(iter);
        Err(Self { errors })
    }

    pub fn combine<T, U>(a: Result<T, Self>, b: Result<U, Self>) -> Result<(T, U), Self> {
        match (a, b) {
            (Ok(a), Ok(b)) => Ok((a, b)),
            (Err(a), Ok(_)) => Err(a),
            (Ok(_), Err(b)) => Err(b),
            (Err(mut a), Err(b)) => {
                a.extend(b);
                Err(a)
            }
        }
    }

    pub fn combine_and_vec<T, U>(
        a: Result<T, Self>,
        b: Result<U, Self>,
        errors: Vec<E>,
    ) -> Result<(T, U), Self> {
        let (a, b) = match Self::combine(a, b) {
            Ok(ok) => ok,
            Err(mut err) => {
                err.extend(errors);
                return Err(err);
            }
        };
        Self::iter_to_result(errors)?;
        Ok((a, b))
    }

    pub fn errors_vec_to_result(errors: Vec<Self>) -> Result<(), Self> {
        if errors.is_empty() {
            return Ok(());
        }

        let mut errors_iter = errors.into_iter();

        let Some(first_errors) = errors_iter.next() else {
            // no errors
            return Ok(());
        };

        let mut result = first_errors;
        for errors in errors_iter {
            result.extend(errors);
        }
        Err(result)
    }

    pub fn vec_result<T>(vec: Vec<Result<T, E>>) -> Result<Vec<T>, Self> {
        let mut ok_result = Vec::new();
        let mut err_result = Vec::new();
        for element in vec {
            match element {
                Ok(ok) => ok_result.push(ok),
                Err(err) => err_result.push(err),
            }
        }
        Self::iter_to_result(err_result).map(|_| ok_result)
    }

    pub fn flat_single_result<T>(vec: Vec<Result<T, E>>) -> Result<Vec<T>, Self> {
        let vec = vec
            .into_iter()
            .map(|element| element.map_err(|err| Self::single(err)))
            .collect();
        Self::flat_result(vec)
    }

    pub fn flat_result<T>(vec: Vec<Result<T, Self>>) -> Result<Vec<T>, Self> {
        let mut ok_result = Vec::new();
        let mut err_result = Vec::new();
        for element in vec {
            match element {
                Ok(ok) => ok_result.push(ok),
                Err(err) => err_result.extend(err.into_iter()),
            }
        }
        Self::iter_to_result(err_result).map(|_| ok_result)
    }

    pub fn add_error(&mut self, error: E) {
        self.errors.push(error);
    }

    pub fn into_errors(self) -> Vec1<E> {
        self.errors
    }

    pub fn convert_inner<F: Error>(self) -> Errors<F>
    where
        E: std::convert::Into<F>,
    {
        Errors::<F> {
            errors: self.errors.mapped(Into::into),
        }
    }
}

impl<E: Error> From<E> for Errors<E> {
    fn from(error: E) -> Self {
        Errors::single(error)
    }
}

impl<E: Error> IntoIterator for Errors<E> {
    type Item = E;

    type IntoIter = <Vec1<E> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<E: Error> Extend<E> for Errors<E> {
    fn extend<T: IntoIterator<Item = E>>(&mut self, iter: T) {
        self.errors.extend(iter);
    }
}

impl<E: Error> Extend<Errors<E>> for Errors<E> {
    fn extend<T: IntoIterator<Item = Errors<E>>>(&mut self, iter: T) {
        for errors in iter {
            self.errors.extend(errors);
        }
    }
}
