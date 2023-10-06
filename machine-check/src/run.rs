use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::anyhow;
use log::{debug, info, warn};
use machine_check_exec_prepare::Preparation;
use machine_check_lib::{create_abstract_machine, write_machine};
use syn::{parse_quote, Item, ItemFn};
use tempdir::TempDir;

pub(super) fn run(args: super::Args) -> Result<(), anyhow::Error> {
    Runner { args }.work()
}

struct Runner {
    args: super::Args,
}

impl Runner {
    fn work(&self) -> Result<(), anyhow::Error> {
        let system_path = Path::new(&self.args.system_path);

        info!("Transcribing the system into a machine.");
        let cwd = std::env::current_dir()?;
        debug!("Current working directory is {:?}.", cwd);
        debug!("The system path is {:?}.", system_path);

        let btor2_file = match File::open(system_path) {
            Ok(file) => file,
            Err(err) => return Err(anyhow!("Cannot open input file {:?}: {}", system_path, err)),
        };

        let concrete_machine: syn::File = syn::parse2(btor2rs::translate_file(btor2_file)?)?;
        let mut abstract_machine = create_abstract_machine(&concrete_machine)?;

        // add main function

        let main_fn: ItemFn = parse_quote!(
            fn main() {
                ::machine_check_exec::run::<mark::Machine>()
            }
        );
        abstract_machine.items.push(Item::Fn(main_fn));

        // the machine package directory path can be given
        // we will write the machine into a temporary directory if it is not given
        // do not drop temporary directory too early
        let (machine_package_dir_path, machine_package_temp_dir) = match &self.args.machine_path {
            Some(path) => (path.clone(), None),
            None => {
                let temp_dir = TempDir::new("machine_check_machine_")?;
                (temp_dir.path().to_path_buf(), Some(temp_dir))
            }
        };

        let src_dir_path = machine_package_dir_path.join("src");
        fs::create_dir_all(&src_dir_path)?;
        let main_path = src_dir_path.join("main.rs");

        debug!("Writing the machine into file {:?}.", main_path);
        write_machine("abstract", &abstract_machine, main_path.as_path())?;

        info!("Building a machine verifier.");
        let artifact_path = self.build_machine(&machine_package_dir_path, main_path.as_path())?;

        info!("Executing the machine verifier.");

        self.execute_machine(&artifact_path)?;

        // warn on error to close the temporary directory, it is not critical
        if let Some(temp_dir) = machine_package_temp_dir {
            if let Err(err) = temp_dir.close() {
                warn!(
                    "Could not close temporary directory for machine: {:#?}",
                    err
                );
            }
        }

        Ok(())
    }

    fn build_machine(
        &self,
        machine_package_dir_path: &Path,
        main_path: &Path,
    ) -> Result<PathBuf, anyhow::Error> {
        let machine_out_dir_path = machine_package_dir_path.join("out");
        fs::create_dir_all(&machine_out_dir_path)?;

        // use rustc if there is preparation, use cargo if there is no preparation
        let (is_rustc, mut command) = match &self.args.preparation_path {
            Some(preparation_path) => {
                // read the preparation definition file
                let preparation_file_path = preparation_path.join("preparation.json");
                debug!(
                    "Reading preparation definition file {:?}.",
                    preparation_file_path
                );
                let preparation_string = std::fs::read_to_string(preparation_file_path)?;
                let preparation: Preparation = serde_json::from_str(preparation_string.as_str())?;

                let mut command = Command::new("rustc");
                command
                    .arg(main_path)
                    .arg("--out-dir")
                    .arg(machine_out_dir_path)
                    .args(preparation.target_build_args);
                (true, command)
            }
            None => {
                todo!();
            }
        };
        debug!("Executing build command {:?}.", command);

        // build with piped
        let build_output = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !build_output.status.success() {
            info!(
                "Build stdout:\n{}\n",
                String::from_utf8(build_output.stdout)?
            );
            info!(
                "Build stderr:\n{}\n",
                String::from_utf8(build_output.stderr)?
            );
            return Err(anyhow!("Build was not successful"));
        }

        let mut executable_path: Option<String> = None;
        // parse output
        if is_rustc {
            let stderr = String::from_utf8(build_output.stderr)?;
            for line in stderr.lines() {
                let hash_map: HashMap<String, String> = serde_json::from_str(line)?;
                if let (Some(artifact), Some(emit)) =
                    (hash_map.get("artifact"), hash_map.get("emit"))
                {
                    if emit == "link" {
                        // this is the executable
                        executable_path = Some(artifact.clone());
                    }
                }
            }
        } else {
            todo!();
        }
        let Some(artifact_path) = executable_path else {
            return Err(anyhow!("Build generated no executable"));
        };
        Ok(PathBuf::from(artifact_path))
    }

    fn execute_machine(&self, artifact_path: &Path) -> Result<(), anyhow::Error> {
        let mut command = Command::new(artifact_path);

        // forward batch
        if self.args.batch {
            command.arg("--batch");
        }
        // forward property
        if let Some(property) = &self.args.property {
            command.arg("--property").arg(property);
        }

        // the machine executable logs on stderr and gives us the result on stdout
        // pipe stdout and inherit stderr
        command.stdout(Stdio::piped()).stderr(Stdio::inherit());

        let exec_output = command.output().unwrap();

        if !exec_output.status.success() {
            return Err(anyhow!("Execution was not successful"));
        }

        // TODO: parse stdout to determine the result

        Ok(())
    }
}
