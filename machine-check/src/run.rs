use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::anyhow;
use log::info;
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

        info!("Creating a machine for system {:?}.", system_path);

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

        let machine_package_dir = TempDir::new("machine_check_machine_").unwrap();
        let machine_package_dir_path = machine_package_dir.path();
        let src_dir_path = machine_package_dir.path().join("src");
        fs::create_dir_all(&src_dir_path)?;
        let main_path = src_dir_path.join("main.rs");

        info!("Writing the machine to file {:?}.", main_path);
        write_machine("abstract", &abstract_machine, main_path.as_path())?;

        info!("Building the machine.");
        let artifact_path = self.build_machine(machine_package_dir_path, main_path.as_path())?;

        info!("Executing the machine.");

        self.execute_machine(&artifact_path)?;

        Ok(())
    }

    fn build_machine(
        &self,
        machine_package_dir_path: &Path,
        main_path: &Path,
    ) -> Result<PathBuf, anyhow::Error> {
        let preparation_string =
            match std::fs::read_to_string("./resources/exec-build/preparation.json") {
                Ok(s) => s,
                Err(err) => return Err(anyhow!("Could not read preparation file: {:#?}", err)),
            };
        let out_dir_path = machine_package_dir_path.join("out");
        fs::create_dir_all(&out_dir_path)?;

        let preparation: Preparation = serde_json::from_str(preparation_string.as_str())?;
        let mut string_args = vec![
            String::from("--edition=2021"),
            String::from("--error-format=json"),
            String::from("--json=artifacts"),
            String::from("--crate-type"),
            String::from("bin"),
            String::from("-C"),
            String::from("opt-level=3"),
            String::from("-C"),
            String::from("embed-bitcode=no"),
            String::from("-C"),
            String::from("strip=symbols"),
        ];
        string_args.extend(preparation.target_build_args);

        let build_output = Command::new("rustc")
            .arg(main_path)
            .args(string_args)
            .arg("--out-dir")
            .arg(out_dir_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        if !build_output.status.success() {
            println!(
                "Build stdout:\n{}\n",
                String::from_utf8(build_output.stdout)?
            );
            println!(
                "Build stderr:\n{}\n",
                String::from_utf8(build_output.stderr)?
            );
            return Err(anyhow!("Build was not successful"));
        }

        let mut artifact_path: Option<String> = None;
        let stderr = String::from_utf8(build_output.stderr)?;
        for line in stderr.lines() {
            let hash_map: HashMap<String, String> = serde_json::from_str(line)?;
            if let (Some(artifact), Some(emit)) = (hash_map.get("artifact"), hash_map.get("emit")) {
                if emit == "link" {
                    // this is the executable
                    artifact_path = Some(artifact.clone());
                }
            }
        }
        let Some(artifact_path) = artifact_path else {
        panic!("Build generated no artifact");
    };
        Ok(PathBuf::from(artifact_path))
    }

    fn execute_machine(&self, artifact_path: &Path) -> Result<(), anyhow::Error> {
        // the machine executable logs on stderr and gives us the result on stdout
        // pipe stdout and inherit stderr

        let exec_output = Command::new(artifact_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .unwrap();

        if !exec_output.status.success() {
            return Err(anyhow!("Execution was not successful"));
        }

        // TODO: parse stdout to determine the result

        Ok(())
    }
}
