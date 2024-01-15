use cargo_metadata::camino::Utf8PathBuf;
use log::debug;
use std::fs::{self};
use syn::{parse_quote, Item, ItemFn};
use tempdir::TempDir;

use crate::{translate, CheckError};
use machine_check_machine::{create_abstract_machine, write_machine};

use super::Verify;

impl Verify {
    pub(super) fn translate(&self) -> Result<(Utf8PathBuf, Option<TempDir>), CheckError> {
        // the machine package directory path can be given
        // we will write the machine into a temporary directory if it is not given
        // do not drop temporary directory too early
        let (machine_package_dir_path, machine_package_temp_dir) =
            match &self.verify_args.machine_path {
                Some(path) => (path.clone(), None),
                None => {
                    let temp_dir = TempDir::new("machine_check_machine_")
                        .map_err(CheckError::CreateTempDir)?;
                    let temp_dir_path = temp_dir.path().to_path_buf();
                    let temp_dir_path = Utf8PathBuf::try_from(temp_dir_path.clone())
                        .map_err(|err| CheckError::PathToUtf8(temp_dir_path, err))?;
                    (temp_dir_path, Some(temp_dir))
                }
            };

        let src_dir_path = machine_package_dir_path.join("src");
        fs::create_dir_all(&src_dir_path)
            .map_err(|err| CheckError::CreateDir(src_dir_path.clone(), err))?;
        let main_path = src_dir_path.join("main.rs");

        let concrete_machine: syn::File = translate::translate(&self.verify_args.system_path)?;
        let mut abstract_machine = create_abstract_machine(&concrete_machine)?;

        // add main function

        let main_fn: ItemFn = parse_quote!(
            fn main() {
                ::machine_check_exec::run::<refin::Input, refin::State, refin::Machine>()
            }
        );
        abstract_machine.items.push(Item::Fn(main_fn));

        debug!("Writing the machine into file {:?}.", main_path);
        write_machine(&abstract_machine, &main_path)?;
        Ok((machine_package_dir_path, machine_package_temp_dir))
    }
}
