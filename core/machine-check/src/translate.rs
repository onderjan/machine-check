use camino::Utf8Path;

use crate::CheckError;

mod btor2;

pub fn translate(system_path: &Utf8Path) -> std::result::Result<syn::File, CheckError> {
    let Some(extension) = system_path.extension() else {
        return Err(CheckError::SystemType(String::from("(no extension)")));
    };

    match extension {
        "btor2" => btor2::translate(system_path),
        _ => Err(CheckError::SystemType(format!("{:?}", extension))),
    }
}
