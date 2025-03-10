use log::info;
use mck::concr::FullMachine;
use std::{sync::Arc, time::Instant};

use crate::shared::{
    snapshot::log::{MessageType, StepMessage},
    BackendStatus, Request, Response,
};

use super::{workspace::WorkspaceProperty, Backend};

pub fn command<M: FullMachine>(backend: &Backend<M>, request: Request) -> Response {
    let mut stats_guard = backend
        .stats
        .lock()
        .expect("The stats should not be poisoned");

    if stats_guard.running {
        // something is running, dismiss the command and signify it

        if !matches!(request, Request::Query) {
            info!("Running, dismissing {:?}", request);
        }
        return Response {
            backend_status: BackendStatus::Running,
            snapshot: None,
        };
    }

    info!("Processing {:?}", request);

    // the backend is waiting for a command, lock the workspace and execute

    let mut workspace_guard = backend
        .workspace
        .lock()
        .expect("The workspace should not be poisoned");

    let (snapshot, running) = match request {
        Request::Query => (None, false),
        Request::GetContent => (
            Some(workspace_guard.generate_snapshot(&backend.settings)),
            false,
        ),
        Request::Reset => {
            workspace_guard.framework.reset();
            (
                Some(workspace_guard.generate_snapshot(&backend.settings)),
                false,
            )
        }
        Request::AddProperty(prepared_property) => {
            workspace_guard
                .properties
                .push(WorkspaceProperty::new(prepared_property));
            (
                Some(workspace_guard.generate_snapshot(&backend.settings)),
                false,
            )
        }
        Request::RemoveProperty(property_index) => {
            workspace_guard.properties.remove(property_index.0);
            (
                Some(workspace_guard.generate_snapshot(&backend.settings)),
                false,
            )
        }
        Request::Step(step_settings) => {
            let snapshot = workspace_guard.generate_snapshot(&backend.settings);

            let worker_workspace = Arc::clone(&backend.workspace);
            let worker_stats = Arc::clone(&backend.stats);

            // signify that the backend is running
            stats_guard.running = true;

            std::thread::spawn(move || {
                info!("Starting stepping.");
                // we will acquire the workspace guard and perform the work
                {
                    let mut workspace_guard = worker_workspace
                        .lock()
                        .expect("The workspace should not be poisoned");

                    info!("Acquired the workspace guard.");

                    let start_instant = Instant::now();
                    let num_refinements = workspace_guard.framework.multi_step_verification(
                        &step_settings.selected_property,
                        false,
                        step_settings.max_refinements,
                    );
                    let duration = start_instant.elapsed();

                    workspace_guard
                        .log
                        .add_message(MessageType::Step(StepMessage {
                            num_refinements,
                            duration,
                        }));
                }
                // we performed the work, acquire the stats guard and signify we are done
                {
                    info!("Acquiring the stats guard.");

                    // acquire the stats guard and signify completion
                    let mut stats_guard = worker_stats
                        .lock()
                        .expect("The stats should not be poisoned");

                    stats_guard.running = false;
                }
                info!("Stepping ended.");
            });
            (Some(snapshot), true)
        }
    };

    // drop both guards, if a thread was spawned, it can start working
    drop(workspace_guard);
    drop(stats_guard);

    Response {
        backend_status: if running {
            BackendStatus::Running
        } else {
            BackendStatus::Waiting
        },
        snapshot,
    }
}
