use camino::Utf8Path;

use crate::CheckError;

mod btor2;

pub fn transcribe(system_path: &Utf8Path) -> std::result::Result<syn::File, CheckError> {
    // TODO: identify system by extension
    btor2::transcribe(system_path)
}
