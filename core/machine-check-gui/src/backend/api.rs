use log::info;
use mck::concr::FullMachine;
use std::{
    ops::ControlFlow,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::shared::{
    snapshot::log::{MessageType, StepMessage, StepStatus},
    BackendStatus, Request, Response, StepSettings,
};

use super::{
    workspace::{Workspace, WorkspaceProperty},
    Backend, BackendStats,
};

pub fn command<M: FullMachine>(backend: &Backend<M>, request: Request) -> Response {
    let mut stats_guard = backend
        .stats
        .lock()
        .expect("The stats should not be poisoned");

    if stats_guard.running {
        // something is running, dismiss the command and signify it

        if matches!(request, Request::Cancel) {
            stats_guard.should_cancel = true;
            info!("Received a cancellation request");
        } else if !matches!(request, Request::Query) {
            info!("Running, dismissing {:?}", request);
        }

        return Response {
            backend_status: backend_status(&stats_guard),
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
        Request::Cancel => {
            // nothing to cancel
            (
                Some(workspace_guard.generate_snapshot(&backend.settings)),
                false,
            )
        }
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

            let workspace_arc = Arc::clone(&backend.workspace);
            let stats_arc = Arc::clone(&backend.stats);

            // scratch cancellations and signify that the backend is running
            stats_guard.should_cancel = false;
            stats_guard.running = true;

            std::thread::spawn(move || backend_step(workspace_arc, stats_arc, step_settings));
            (Some(snapshot), true)
        }
    };

    Response {
        backend_status: if running {
            BackendStatus::Running
        } else {
            BackendStatus::Waiting
        },
        snapshot,
    }
}

fn backend_status(stats: &BackendStats) -> BackendStatus {
    if stats.running {
        if stats.should_cancel {
            BackendStatus::Cancelling
        } else {
            BackendStatus::Running
        }
    } else {
        BackendStatus::Waiting
    }
}

fn backend_step<M: FullMachine>(
    workspace_arc: Arc<Mutex<Workspace<M>>>,
    stats_arc: Arc<Mutex<BackendStats>>,
    step_settings: StepSettings,
) {
    {
        info!("Starting stepping.");
        // we will acquire the workspace guard and perform the work
        {
            let mut workspace_guard = workspace_arc
                .lock()
                .expect("The workspace should not be poisoned");

            info!("Acquired the workspace guard.");

            let start_instant = Instant::now();

            // multi-step with possible cancellation between steps

            let mut num_refinements = 0;
            let mut cancelled = false;
            loop {
                // if the maximum number of refinements is given, stop stepping when it is reached
                if let Some(max_refinements) = step_settings.max_refinements {
                    if num_refinements >= max_refinements {
                        break;
                    }
                }

                // if cancellation was requested, stop stepping
                let should_cancel = {
                    let stats_guard = stats_arc.lock().expect("The stats should not be poisoned");
                    stats_guard.should_cancel
                };

                if should_cancel {
                    info!("Cancelling stepping");
                    cancelled = true;
                    break;
                }

                // stop stepping when we are done
                // TODO: implement inherent assumption for the GUI
                if let ControlFlow::Break(_) = workspace_guard
                    .framework
                    .step_verification(&step_settings.selected_property, false)
                {
                    break;
                }

                num_refinements += 1;
            }
            let duration = start_instant.elapsed();

            workspace_guard
                .log
                .add_message(MessageType::Step(StepMessage {
                    status: if cancelled {
                        StepStatus::Cancelled
                    } else {
                        StepStatus::Completed
                    },
                    num_refinements,
                    duration,
                }));
        }
        // we performed the work, acquire the stats guard and signify we are done
        {
            info!("Acquiring the stats guard.");

            // acquire the stats guard and signify completion
            let mut stats_guard = stats_arc.lock().expect("The stats should not be poisoned");

            stats_guard.running = false;
        }
        info!("Stepping ended.");
    }
}
