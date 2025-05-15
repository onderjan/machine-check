use proc_macro2::Span;

use crate::BackwardError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ErrorType {
    #[error("machine-check: Cannot parse module without content")]
    ModuleWithoutContent,

    #[error("machine-check error: Unknown macro")]
    UnknownMacro,
    #[error("{0}")]
    MacroError(String),
    #[error("{0}")]
    MacroParseError(syn::Error),
    #[error("machine-check: {0}")]
    UnsupportedConstruct(String),
    #[error("machine-check: {0}")]
    IllegalConstruct(String),
    #[error("machine-check: Could not infer variable type")]
    InferenceFailure,
    #[error("machine-check (concrete conversion): {0}")]
    ConcreteConversionError(String),
    #[error("machine-check (forward conversion): {0}")]
    ForwardConversionError(String),
    #[error("machine-check (backward conversion): {0}")]
    BackwardConversionError(String),

    #[error("machine-check internal error (SSA translation): {0}")]
    SsaInternal(String),
    #[error("machine-check internal error (forward translation): {0}")]
    ForwardInternal(String),
    #[error("machine-check internal error (backward translation): {0}")]
    BackwardInternal(String),
    #[error("machine-check internal error (rules): {0}")]
    RulesInternal(String),
}

impl From<BackwardError> for MachineError {
    fn from(error: BackwardError) -> Self {
        MachineError {
            ty: ErrorType::BackwardConversionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{span:?}: {ty}")]
pub struct MachineError {
    pub ty: ErrorType,
    pub span: Span,
}

impl MachineError {
    pub fn new(ty: ErrorType, span: Span) -> Self {
        Self { ty, span }
    }
}

pub struct MachineErrors {
    errors: Vec<MachineError>,
}

impl MachineErrors {
    pub fn single(error: MachineError) -> MachineErrors {
        MachineErrors {
            errors: vec![error],
        }
    }

    pub fn combine_results<T, U>(
        a: Result<T, MachineErrors>,
        b: Result<U, MachineErrors>,
    ) -> Result<(T, U), MachineErrors> {
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

    pub fn errors_vec_to_result(vec: Vec<MachineErrors>) -> Result<(), MachineErrors> {
        if vec.is_empty() {
            return Ok(());
        }

        let mut errors = Vec::new();
        for element in vec {
            errors.extend(element.errors);
        }
        Err(MachineErrors { errors })
    }

    pub fn vec_result<T>(vec: Vec<Result<T, MachineError>>) -> Result<Vec<T>, MachineErrors> {
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
        Err(MachineErrors { errors: err_result })
    }

    pub fn flat_single_result<T>(
        vec: Vec<Result<T, MachineError>>,
    ) -> Result<Vec<T>, MachineErrors> {
        let vec = vec
            .into_iter()
            .map(|element| element.map_err(MachineErrors::single))
            .collect();
        Self::flat_result(vec)
    }

    pub fn flat_result<T>(vec: Vec<Result<T, MachineErrors>>) -> Result<Vec<T>, MachineErrors> {
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
        Err(MachineErrors { errors: err_result })
    }

    pub fn add_error(&mut self, error: MachineError) {
        self.errors.push(error);
    }

    pub fn extend(&mut self, other: MachineErrors) {
        self.errors.extend(other.errors);
    }

    pub fn into_errors(self) -> Vec<MachineError> {
        self.errors
    }
}

impl From<MachineError> for MachineErrors {
    fn from(error: MachineError) -> Self {
        MachineErrors::single(error)
    }
}
