use std::{ffi::OsStr, path::Path, sync::RwLock};

use business::Business;
use log::error;
use machine_check_common::ExecError;
use machine_check_exec::Framework;
use mck::concr::FullMachine;
use window::Window;

mod business;
mod window;

const FAVICON_ICO: &[u8] = include_bytes!("../content/favicon.ico");

pub fn run<M: FullMachine>(system: M) -> Result<(), ExecError> {
    // TODO: allow setting custom titles instead of relying on the binary name
    let exec_name = std::env::current_exe()
        .ok()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_stem)
        .and_then(OsStr::to_str)
        .map(String::from)
        .unwrap_or(String::from("Unknown executable"));

    let abstract_system = <M::Abstr as mck::abstr::Abstr<M>>::from_concrete(system);
    // create the business logic
    let business = RwLock::new(Business::<M>::new(
        Framework::new(
            abstract_system,
            machine_check_exec::VerificationType::Inherent,
            &machine_check_exec::Strategy {
                naive_inputs: false,
                use_decay: false,
            },
        ),
        exec_name.clone(),
    ));
    let response_fn = move |request| Business::get_http_response(&business, request);

    // initialise the GUI
    let gui = match Window::new(response_fn, &exec_name) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Cannot create GUI: {}", err);
            return Err(ExecError::GuiError(err.to_string()));
        }
    };
    // run the GUI, never returns
    gui.run()
}
