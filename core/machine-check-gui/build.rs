use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use tempfile::TempDir;
use tinyjson::JsonValue;
use toml_edit::Table;

/// A warning macro that uses Cargo warning, since stdout of build.rs is processed by Cargo.
///
/// The newlines must be escaped as Cargo will take a newline to signify end of the command.
macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*).replace('\n', "\\n"))
    }
}

/// A debug macro that uses Cargo warning only if the corresponding feature is enabled.
macro_rules! debug {
    ($($tokens: tt)*) => {
        #[cfg(feature = "build-log-debug")]
        { println!("cargo:warning=debug: {}", format!($($tokens)*).replace('\n', "\\n")); }
        #[cfg(not(feature = "build-log-debug"))]
        { let _ = format!($($tokens)*);}
    }
}

/// Whether to build the WebAssembly frontend.
///
///
const ENV_BUILD_FRONTEND: &str = "MACHINE_CHECK_GUI_BUILD_FRONTEND";

/// Whether to prepare the WebAssembly frontend artefacts for distribution.
///
///
const ENV_PREPARE_FRONTEND: &str = "MACHINE_CHECK_GUI_PREPARE_FRONTEND";

/// Name of the environment variable that determines where the frontend will be built.
///
/// If not defined, a temporary directory will be used.
///
/// It is useful to set this variable when working on the machine-check-gui itself.
/// allowing compilation speedup due to caching. The path must not be a member
/// of a higher directory level workspace, since the frontend package will be its
/// own workspace and Cargo gives an error if there is a workspace within a workspace.
///
/// In the standard case when we are not working on machine-check-gui, the environment variable
/// will be unset and we will run this build.rs (and frontend build) only once after each cleanup,
/// so the compilation speedup is not important. We will create a temporary directory for
/// the frontend build, and delete it afterwards.
const ENV_WASM_DIR: &str = "MACHINE_CHECK_GUI_WASM_DIR";

/// Name of the environment variable that determines whether we should postpone build errors.
///
/// If not defined, build errors will not be postponed. If defined, must be 'true' or 'false'
/// with the letter case ignored, otherwise, build.rs will panic.
///
/// Postponement should be usually disabled, but in case we are working on machine-check-gui itself,
/// we want build.rs to succeed so that machine-check-gui will be compiled traditionally afterwards,
/// prompting most errors to correctly show up in rust-analyzer.
/// We make sure that the frontend build produced the frontend WASM in lib.rs by including,
/// so an error should always be produced if the frontend build fails.
const ENV_POSTPONE_ERRORS: &str = "MACHINE_CHECK_GUI_POSTPONE_ERRORS";

const ENV_VARIABLES: [&str; 2] = [ENV_WASM_DIR, ENV_POSTPONE_ERRORS];

/// The main function. Simply calls the run function and and panics on error.
fn main() {
    if let Err(err) = run() {
        panic!("Error building WASM frontend: {}", err);
    }
}

/// The run function.
///
/// Either builds the fronend or, by default, arranges the sources and checks if they match
/// the prepared WebAssembly files. This alleviates the need for every user to install
/// the appropriate WebAssembly build tools.
fn run() -> anyhow::Result<()> {
    let prepare = bool_env(ENV_PREPARE_FRONTEND, false)?;
    let arrangement = arrange(prepare)?;
    if bool_env(ENV_BUILD_FRONTEND, false)? {
        build(&arrangement)?;
    } else {
        copy_prebuilt_wasm(arrangement.hex_hash.clone())?;
    }

    // Close and remove the temporary directory if we created it.
    if let Some(frontend_package_tempdir) = arrangement.frontend_package_tempdir {
        if let Err(err) = frontend_package_tempdir.close() {
            // just warn if we did not succeed
            warn!("Could not close WASM temporary directory: {}", err)
        }
    };
    Ok(())
}

fn copy_prebuilt_wasm(hex_hash: String) -> anyhow::Result<()> {
    // Change the current working directory to the cargo manifest directory of our package.
    let this_package_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(this_package_dir)
        .expect("Should be able to move to manifest directory");

    // Ensure that the prebuilt WASM binaries match the source code by comparing the hash to the arrangement.
    let prebuilt_wasm_dir = this_package_dir.join(PREBUILT_WASM_DIR);
    if !(std::fs::metadata(&prebuilt_wasm_dir).is_ok_and(|metadata| metadata.is_dir())) {
        return Err(anyhow!(
            "Cannot use prebuilt frontend WebAssembly as the folder does not exist."
        ));
    }
    let prebuilt_hex_hash =
        fs::read_to_string(prebuilt_wasm_dir.join(HASH_FILE_NAME)).map_err(|err| {
            anyhow!(
                "Could not read the file of prebuilt WebAssembly frontend: {}",
                err
            )
        })?;
    if hex_hash != prebuilt_hex_hash {
        return Err(anyhow!(
            "The prebuilt WebAssembly frontend does not match the source code"
        ));
    }

    // We now have everything we need. Delete the previous artefacts.
    let artifact_dir = this_package_dir.join(ARTIFACT_DIR);
    let _ = fs::remove_dir_all(&artifact_dir);

    // Copy the prebuilt WASM binaries to the artefact directory.
    copy_dir_all(prebuilt_wasm_dir, artifact_dir)
        .map_err(|err| anyhow!("Cannot copy the prebuilt frontend directory: {err}"))?;

    Ok(())
}

/// Build the WebAssembly frontend.
///
/// It does not seem to be currently possible to build cross-target directly in one cargo build call.
/// We also cannot call cargo build on the current package using a different target as this directory
/// is already locked by cargo (and we would wait for the lock forever).
///
/// So we need to be clever, arrange and build the frontend somewhere else. This function gets the
/// arrangement data and and builds.
fn build(arrangement: &Arrangement) -> anyhow::Result<()> {
    // Compile the frontend package, only warning if we should postpone errors.
    match compile_frontend_package(arrangement) {
        Ok(None) => {}
        Ok(Some(postponed)) => {
            warn!(
                "WASM frontend build failed (postponing error): {}",
                postponed
            )
        }
        Err(err) => return Err(anyhow!("Build failed: {}", err)),
    }

    if arrangement.prepare {
        // copy the compiled content to the prebuilt WASM directory.
        let prebuilt_wasm_dir = arrangement.this_package_dir.join(PREBUILT_WASM_DIR);
        let artifact_dir = arrangement.this_package_dir.join(ARTIFACT_DIR);
        let _ = fs::remove_dir_all(&prebuilt_wasm_dir);

        // Copy the prebuilt WASM binaries to the artefact directory.
        copy_dir_all(artifact_dir, prebuilt_wasm_dir)
            .map_err(|err| anyhow!("Cannot copy the prebuilt frontend directory: {err}"))?;
        warn!("Frontend WebAssembly prepared for deployment.")
    }

    Ok(())
}

struct Arrangement {
    prepare: bool,
    this_package_dir: PathBuf,
    frontend_package_tempdir: Option<TempDir>,
    frontend_package_dir: PathBuf,
    artifact_dir: PathBuf,
    postpone_build_errors: bool,
    hex_hash: String,
}

fn arrange(prepare: bool) -> anyhow::Result<Arrangement> {
    // Change the current working directory to the cargo manifest directory of our package (not the workspace).
    let this_package_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(this_package_dir)
        .expect("Should be able to move to manifest directory");

    // Only rerun build.rs if frontend-related files, directories, or environment variables change.
    COPY_DIRECTORIES
        .into_iter()
        .chain(COPY_FILES)
        .chain(SPECIAL_FILES)
        .for_each(|path| {
            println!("cargo::rerun-if-changed={}", path);
        });
    for env_variable in ENV_VARIABLES {
        println!("cargo::rerun-if-env-changed={}", env_variable);
    }

    // Determine if we should postpone build errors for the frontend.
    //
    // In the standard case when we are not working on machine-check-gui, no errors should be present
    // at all, so we can just safely not postpone build errors.

    let postpone_build_errors = bool_env(ENV_POSTPONE_ERRORS, false)?;

    // Get the location where frontend WASM package should be from an environment variable.
    let (frontend_package_dir, frontend_package_tempdir) = match std::env::var_os(ENV_WASM_DIR) {
        Some(wasm_dir) => {
            // canonicalize the WASM directory using the current working directory, which is the cargo manifest directory
            fs::create_dir_all(wasm_dir.clone()).expect("Should be able to move to WASM directory");
            let wasm_dir =
                fs::canonicalize(wasm_dir).expect("Should be able to canonicalize WASM directory");

            (wasm_dir, None)
        }
        None => {
            // Create the temporary directory.
            let tempdir = tempfile::TempDir::with_prefix("wasm_crate_")
                .expect("Should be able to create a temporary directory");

            (tempdir.path().to_path_buf(), Some(tempdir))
        }
    };

    debug!("Frontend package directory: {:?}", frontend_package_dir);

    // Get the package and workspace Cargo.toml path. This works as we are still in the cargo manifest directory.
    let package_toml_path = cargo_toml_path(false)?;
    let workspace_toml_path = cargo_toml_path(true)?;

    // If the package and workspace Cargo.toml path is the same, the package is the root of the workspace.
    let mut workspace_toml_path = if package_toml_path != workspace_toml_path {
        Some(workspace_toml_path)
    } else {
        None
    };

    // If we are preparing for deployment, ignore workspace Cargo.toml.
    if prepare {
        warn!("Preparing the frontend WebAssembly for deployment. Make sure you know what you are doing.");
        if workspace_toml_path.is_some() {
            warn!("Overriding workspace build due to deployment preparation.");
            workspace_toml_path = None;
        }
    }

    // We now have everything we need. Delete the previous artefacts.
    let artifact_dir = this_package_dir.join(ARTIFACT_DIR);
    let _ = fs::remove_dir_all(&artifact_dir);

    // Arrange the frontend package.

    arrange_frontend_package(
        this_package_dir,
        &frontend_package_dir,
        package_toml_path,
        workspace_toml_path,
    )
    .map_err(|err| anyhow!("Package preparation failed: {}", err))?;

    // Get the frontend package directory hash.
    let hex_hash = directory_hex_hash(String::from(
        frontend_package_dir
            .to_str()
            .expect("Frontend package dir should be Unicode"),
    ))?;

    Ok(Arrangement {
        prepare,
        this_package_dir: this_package_dir.to_path_buf(),
        frontend_package_tempdir,
        frontend_package_dir: frontend_package_dir.to_path_buf(),
        artifact_dir: artifact_dir.to_path_buf(),
        postpone_build_errors,
        hex_hash,
    })
}

const COPY_DIRECTORIES: [&str; 2] = ["src/shared", "src/frontend"];
const COPY_FILES: [&str; 2] = ["src/frontend.rs", "src/shared.rs"];
const LIB_RS: &str = "src/lib.rs";
const CARGO_TOML: &str = "Cargo.toml";
const RUST_TOOLCHAIN_TOML: &str = "rust-toolchain.toml";
const SPECIAL_FILES: [&str; 2] = [LIB_RS, CARGO_TOML];

const ARTIFACT_DIR: &str = "content/wasm";

const PREBUILT_WASM_DIR: &str = "prebuilt_wasm";

const HASH_FILE_NAME: &str = "hash.hex";

/// Arrange the frontend package directory for building.
///
/// After cleaning the frontend package directory, we will copy the frontend sources to the frontend
/// package directory appropriately, and add a custom lib.rs to ensure only the frontend is compiled.
///
/// For Cargo.toml, we will take our own machine-check-gui Cargo.toml as a starting point.
/// Since the frontend package directory may or may not be the member of the same (or even another)
/// workspace, we will add a [workspace] entry to force Cargo to error if the frontend package directory
/// is a member of another workspace. To ensure we still have [patch] and [profile] from the workspace
/// machine-check-gui is in, we will process its Cargo.toml (if it exists) and move them. We will panic
/// on the deprecated [replace] in workspace, and also move [workspace] dependencies and lints.
///
/// While doing this, we must convert the local paths in machine-check-gui [dependencies] and workspace
/// [patch] and [workspace.dependencies], ensuring they are absolute. We will simply do this by taking
/// the canonical path while in the appropriate working directory.
///
/// This should handle a reasonable amount of Cargo.toml configuration, especially when building
/// standalone and when building the official machine-check repository.
///
/// This function can change the working directory.
fn arrange_frontend_package(
    this_package_dir: &Path,
    frontend_package_dir: &Path,
    package_toml_path: PathBuf,
    workspace_toml_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    if this_package_dir.join(RUST_TOOLCHAIN_TOML).exists() {
        return Err(anyhow!(
            "A rust-toolchain.toml in the package directory is not supported"
        ));
    }

    // Copy the appropriate directories and files to the WASM package directory.
    for copy_directory in COPY_DIRECTORIES {
        copy_dir_all(
            this_package_dir.join(copy_directory),
            frontend_package_dir.join(copy_directory),
        )?;
    }

    for copy_file in COPY_FILES {
        fs::copy(
            this_package_dir.join(copy_file),
            frontend_package_dir.join(copy_file),
        )?;
    }

    // Handle lib.rs specially: just declare the shared and frontend.
    fs::write(
        frontend_package_dir.join(LIB_RS),
        "pub mod shared; mod frontend;\n",
    )?;

    // Handle the package Cargo.toml specially.
    let cargo_toml = fs::read_to_string(this_package_dir.join(CARGO_TOML))?;
    let mut cargo_toml: toml_edit::DocumentMut = cargo_toml.parse()?;

    // If there is [workspace], [patch], or [replace] in the package Cargo.toml, return an error,
    // so that we do not have to do complicated patching.
    if cargo_toml.contains_key("workspace") {
        return Err(anyhow!(
            "Workspace entry in package Cargo.toml not supported"
        ));
    }
    if cargo_toml.contains_key("patch") {
        return Err(anyhow!("Patch entry in package Cargo.toml not supported"));
    }
    if cargo_toml.contains_key("replace") {
        return Err(anyhow!("Replace entry in package Cargo.toml not supported"));
    }

    // Update the package name to prevent potential confusion.
    // Insert the workspace table so it is its own workspace.
    // Also set publish=false as it is just a generated package.

    cargo_toml["package"]["name"] = "machine-check-gui-wasm".into();
    cargo_toml.insert("workspace", toml_edit::Item::Table(Table::new()));
    cargo_toml["publish"] = false.into();

    // Next, canonicalize the paths in the package Cargo.toml.
    // For simplicity, only process [dependencies].
    canonicalize_paths(&package_toml_path, &mut cargo_toml["dependencies"])?;

    let mut patched_repository_package_paths: BTreeMap<String, BTreeMap<String, String>> =
        BTreeMap::new();

    // Adjust using the workspace Cargo.toml if there is one.
    if let Some(workspace_toml_path) = workspace_toml_path {
        // Read and parse the workspace.
        let workspace_toml = fs::read_to_string(&workspace_toml_path)?;
        let workspace_toml: toml_edit::DocumentMut = workspace_toml.parse()?;

        let package_workspace = cargo_toml
            .get_mut("workspace")
            .unwrap()
            .as_table_mut()
            .unwrap();

        // Apply [workspace.dependencies] with adjusted paths if it exists.
        if let Some(workspace_dependencies) = workspace_toml["workspace"].get("dependencies") {
            let mut workspace_dependencies = workspace_dependencies.clone();
            canonicalize_paths(&workspace_toml_path, &mut workspace_dependencies)?;
            package_workspace.insert("dependencies", workspace_dependencies);
        }

        // Just copy over the [workspace.lints] if they exist.
        if let Some(workspace_lints) = workspace_toml["workspace"].get("lints") {
            package_workspace.insert("lints", workspace_lints.clone());
        }

        // Return an error if there is a [replace] section, it is deprecated in favour of [patch] anyway.

        if workspace_toml.contains_key("replace") {
            return Err(anyhow!(
                "Replace entry in workspace Cargo.toml not supported. Consider using patch."
            ));
        }

        // Apply [patch] with adjusted paths if it exists.
        if let Some(patch) = workspace_toml.get("patch") {
            let mut patch = patch.clone();
            // Patches are a list for each repository, so process everything in a loop.
            let Some(patch_table) = patch.as_table_mut() else {
                return Err(anyhow!(
                    "Unexpected non-table workspace patch in {:?}",
                    workspace_toml_path
                ));
            };

            for (repository_key, repository_value) in patch_table.iter_mut() {
                let patched_package_paths =
                    canonicalize_paths(&workspace_toml_path, repository_value)?;
                let entry = patched_repository_package_paths
                    .entry(repository_key.to_string())
                    .or_default();

                entry.extend(patched_package_paths.into_iter());
            }
            debug!("Applying workspace patch to WASM package");
            cargo_toml.insert("patch", patch);
        }

        // Copy workspace rust-toolchain.toml if it exists.
        let workspace_rust_toolchain_toml = workspace_toml_path
            .parent()
            .ok_or(anyhow!("Workspace Cargo.toml should have a parent"))?
            .join(RUST_TOOLCHAIN_TOML);
        if workspace_rust_toolchain_toml.exists() {
            fs::copy(
                workspace_rust_toolchain_toml.clone(),
                frontend_package_dir.join(RUST_TOOLCHAIN_TOML),
            )
            .map_err(|err| anyhow!("Workspace rust-toolchain.toml could not be copied: {}", err))?;
        }
        // rerun the build script if the workspace Cargo.toml or rust-toolchain.toml changes
        cargo_rerun_if_path_changed(&workspace_toml_path)?;
        cargo_rerun_if_path_changed(&workspace_rust_toolchain_toml)?;
    }

    // Ensure build.rs is rebuilt when something is changed in patched dependency paths.

    let dependencies = cargo_toml
        .get("dependencies")
        .ok_or(anyhow!("Expected dependencies in Cargo.toml"))?;
    let dependencies = dependencies
        .as_table()
        .ok_or(anyhow!("Unexpected non-table dependencies in Cargo.toml"))?;

    for (dependency_name, dependency) in dependencies {
        let registry = if let Some(dependency) = dependency.as_table_like() {
            if let Some(registry) = dependency.get("registry") {
                Some(
                    registry
                        .as_str()
                        .ok_or(anyhow!("Unexpected non-string registry in Cargo.toml"))?,
                )
            } else {
                None
            }
        } else {
            None
        };
        let registry = registry.unwrap_or("crates-io");

        if let Some(patched_package_paths) = patched_repository_package_paths.get(registry) {
            if let Some(patched_path) = patched_package_paths.get(dependency_name) {
                debug!(
                    "Ensuring rebuild on repository {} dependency {} patched path {:?}",
                    dependency_name, registry, patched_path
                );
                // rerun the build script if the dependency changes
                println!("cargo::rerun-if-changed={}", patched_path);
            }
        }
    }

    // We have done the important adjustments, write the Cargo.toml to the frontend package directory.
    fs::write(
        frontend_package_dir.join(CARGO_TOML),
        cargo_toml.to_string(),
    )?;

    // Add a wildcard .gitignore to the created WASM package as it is generated.
    fs::write(frontend_package_dir.join(".gitignore"), "*\n")?;

    Ok(())
}

/// Compile the frontend and process it with wasm-bindgen, producing the final artefacts.
///
/// After arranging the frontend package, we will cargo build it with the WASM target, and run
/// wasm-bindgen on the built artefacts afterwards, producing the WASM files into the directory
/// from which they will be served. Before the build, we will empty the directory so that we never get
/// stale artefacts. The artefact existence is checked in machine-check-gui lib.rs, producing an error
/// if they do not exist after this build.rs executes normally.
fn compile_frontend_package(arrangement: &Arrangement) -> anyhow::Result<Option<anyhow::Error>> {
    // Prepare arguments for cargo build.
    // We want the target-dir (where to build to) to be the frontend package target subdirectory.
    let cargo_target_dir: PathBuf = arrangement.frontend_package_dir.join("target");
    let cargo_target_dir_arg = create_equals_arg("target-dir", &cargo_target_dir);

    // Prepare cargo build for the WASM target.
    std::env::set_current_dir(&arrangement.frontend_package_dir)?;
    let mut cargo_build = Command::new(env!("CARGO"));
    cargo_build
        .current_dir(&arrangement.frontend_package_dir)
        .args([
            "build",
            "--package",
            "machine-check-gui-wasm",
            "--target=wasm32-unknown-unknown",
        ]);
    // Set the profile according to the current build profile.
    // TODO: Set the exactly same profile as our build.
    // This currently cannot be done easily due to https://github.com/rust-lang/cargo/issues/11054.
    let profile = std::env::var("PROFILE")
        .map_err(|err| anyhow!("Cannot retrieve build profile: {}", err))?;
    if profile == "debug" {
        // is used normally, no argument needed
    } else if profile == "release" {
        cargo_build.arg("--release");
    } else {
        return Err(anyhow!("Unknown build profile {}", profile));
    };

    cargo_build.arg(cargo_target_dir_arg);
    if let Err(err) = execute_command("cargo build", cargo_build) {
        if arrangement.postpone_build_errors {
            // propagate the error
            return Ok(Some(err));
        } else {
            return Err(err);
        }
    }

    // Prepare arguments for wasm-bindgen.
    let bindgen_out_dir_arg = create_equals_arg("out-dir", &arrangement.artifact_dir);

    // Execute wasm-bindgen to obtain the final WASM with proper bindings.
    std::env::set_current_dir(&arrangement.this_package_dir)?;
    let mut wasm_bindgen = Command::new("wasm-bindgen");
    let target_path = format!(
        "wasm32-unknown-unknown/{}/machine_check_gui_wasm.wasm",
        profile
    );

    wasm_bindgen
        .current_dir(&arrangement.this_package_dir)
        .arg("--target=web")
        .arg(bindgen_out_dir_arg)
        .arg(cargo_target_dir.join(target_path));
    execute_command("wasm-bindgen", wasm_bindgen)?;

    // Write the frontend package directory hash to the artefact directory.
    fs::write(
        arrangement.artifact_dir.join(HASH_FILE_NAME),
        arrangement.hex_hash.clone(),
    )
    .map_err(|err| anyhow!("Cannot write frontend directory hash value: {err}"))?;
    Ok(None)
}

// --- HELPER FUNCTIONS ---

/// Make Cargo.toml paths inside a package list absolute from the given Cargo.toml path.
///
/// This function changes the working directory. Returns a map of package names to
/// corresponding paths that were patched.
fn canonicalize_paths(
    orig_toml_path: &Path,
    package_list: &mut toml_edit::Item,
) -> anyhow::Result<BTreeMap<String, String>> {
    // Set the current directory to the parent of the original TOML file.
    let Some(orig_directory) = orig_toml_path.parent() else {
        return Err(anyhow!("No parent of TOML file path {:?}", orig_toml_path));
    };

    std::env::set_current_dir(orig_directory).map_err(|err| {
        anyhow!(
            "Cannot change the current working directory to {:?}: {}",
            orig_directory,
            err
        )
    })?;

    // Coerce all path values to canonical.
    let Some(package_list) = package_list.as_table_mut() else {
        return Err(anyhow!(
            "Unexpected non-table package list {} (in {:?})",
            package_list,
            orig_toml_path
        ));
    };

    let mut package_name_paths = BTreeMap::new();

    for (package_key, package_value) in package_list.iter_mut() {
        let Some(package_value) = package_value.as_inline_table_mut() else {
            if package_value.is_str() {
                // just a version number, skip
                continue;
            }

            return Err(anyhow!(
                "Unexpected non-inline-table package value for {}: '{}' (in {:?})",
                package_key,
                package_value,
                orig_toml_path
            ));
        };
        for (key, value) in package_value.iter_mut() {
            if key != "path" {
                // only path values need to be adjusted
                continue;
            }
            let package_path = value.as_str().unwrap();
            let canonical_path = fs::canonicalize(package_path)?;
            debug!(
                "Adjusting {} path to canonical {:?}",
                package_key, canonical_path
            );
            let canonical_path_string = canonical_path.to_str().ok_or_else(|| {
                anyhow!("Unexpected non-UTF-8 dependency path: {:?}", canonical_path)
            })?;

            package_name_paths.insert(package_key.to_string(), canonical_path_string.to_string());

            *value = toml_edit::Value::String(toml_edit::Formatted::new(
                canonical_path.into_os_string().into_string().unwrap(),
            ));
        }
    }
    // We are done.
    Ok(package_name_paths)
}

// Return the workspace/package path for the current working directory.
fn cargo_toml_path(workspace: bool) -> anyhow::Result<PathBuf> {
    let mut command = std::process::Command::new(env!("CARGO"));
    command.arg("locate-project");
    if workspace {
        command.arg("--workspace");
    }
    // Ensure we are using the JSON format, so that we are not in danger of trimmed spaces
    // contained in the original path (unlikely, but better safe than sorry).
    command.arg("--message-format=json");
    let output = command
        .output()
        .map_err(|err| anyhow!("Should be executable: {}", err))?;
    if !output.status.success() {
        return Err(anyhow!("Non-success status code {}", output.status));
    }

    let json_value: JsonValue = std::str::from_utf8(&output.stdout)
        .map_err(|err| anyhow!("Output should be UTF-8: {}", err))?
        .parse()
        .map_err(|err| anyhow!("Output should be JSON: {}", err))?;
    let json_object: &std::collections::HashMap<_, _> = json_value
        .get()
        .ok_or(anyhow!("Output should be an object"))?;
    let json_root = json_object
        .get("root")
        .ok_or(anyhow!("Output should have a 'root' element"))?;
    let json_root: &String = json_root
        .get()
        .ok_or(anyhow!("The 'root' element should be a string"))?;

    debug!(
        "Cargo.toml path ({}): {}",
        if workspace { "workspace" } else { "package" },
        json_root
    );

    Ok(PathBuf::from(json_root))
}

/// Copy the whole directory recursively.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for dir_entry in fs::read_dir(src)? {
        let dir_entry = dir_entry?;
        let file_type = dir_entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(dir_entry.path(), dst.as_ref().join(dir_entry.file_name()))?;
        } else {
            fs::copy(dir_entry.path(), dst.as_ref().join(dir_entry.file_name()))?;
        }
    }
    Ok(())
}

/// Execute a command and returns an error on non-successful execution.
fn execute_command(name: &str, mut command: Command) -> anyhow::Result<()> {
    let output = command.output()?;
    if !output.status.success() {
        // cargo should only typically write on stderr
        Err(anyhow!(
            "{} failed, status code: {}\n --- Output:\n{}",
            name,
            output.status,
            String::from_utf8(output.stderr)?
        ))
    } else {
        debug!(
            "{} succeeded, status code: {}\n --- Output:\n{}",
            name,
            output.status,
            String::from_utf8(output.stderr)?
        );
        Ok(())
    }
}

/// Creates an argument of form '--name=path' from the argument name and path.
fn create_equals_arg(arg_name: &str, path: &Path) -> OsString {
    let mut result = OsString::from("--");
    result.push(arg_name);
    result.push("=");
    result.push(path.as_os_str());
    result
}

fn bool_env(name: &str, default: bool) -> anyhow::Result<bool> {
    println!("cargo::rerun-if-env-changed={}", name);
    let val = std::env::var_os(name);
    debug!(
        "Bool env var {} value: {:?} (default {})",
        name, val, default
    );
    let Some(val) = val else { return Ok(default) };
    // ignore case to give build scripts some leeway
    if val.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if val.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(anyhow!("The environment variable '{}' should have a Boolean value (true or false ignoring case).", name))
    }
}

fn directory_hex_hash(absolute_path: String) -> anyhow::Result<String> {
    let hash_tree = merkle_hash::MerkleTree::builder(&absolute_path)
        .hash_names(true)
        .build()
        .map_err(|err| anyhow!("Cannot hash-stamp files of interest: {}", err))?;
    let path_hash = hash_tree.root.item.hash;

    Ok(hex::encode(&path_hash))
}

fn cargo_rerun_if_path_changed(path: &Path) -> anyhow::Result<()> {
    let path_str = path
        .to_str()
        .ok_or(anyhow!("Workspace Cargo.toml should have Unicode path"))?;
    println!("cargo::rerun-if-changed={}", path_str);
    Ok(())
}
