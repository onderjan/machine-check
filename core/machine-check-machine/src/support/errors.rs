use std::error::Error;

pub struct Errors<E: Error> {
    errors: Vec<E>,
}

impl<E: Error> Errors<E> {
    pub fn single(error: E) -> Self {
        Errors {
            errors: vec![error],
        }
    }

    pub fn combine_results<T, U>(a: Result<T, Self>, b: Result<U, Self>) -> Result<(T, U), Self> {
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

    pub fn errors_vec_to_result(vec: Vec<Self>) -> Result<(), Self> {
        if vec.is_empty() {
            return Ok(());
        }

        let mut errors = Vec::new();
        for element in vec {
            errors.extend(element.errors);
        }
        Err(Self { errors })
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
        if err_result.is_empty() {
            return Ok(ok_result);
        }
        Err(Self { errors: err_result })
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
                Err(err) => err_result.extend(err.into_errors()),
            }
        }
        if err_result.is_empty() {
            return Ok(ok_result);
        }
        Err(Errors { errors: err_result })
    }

    pub fn add_error(&mut self, error: E) {
        self.errors.push(error);
    }

    pub fn extend(&mut self, other: Self) {
        self.errors.extend(other.errors);
    }

    pub fn into_errors(self) -> Vec<E> {
        self.errors
    }
}

impl<E: Error> From<E> for Errors<E> {
    fn from(error: E) -> Self {
        Errors::single(error)
    }
}
