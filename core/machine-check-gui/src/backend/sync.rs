use std::{
    ops::ControlFlow,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender, TrySendError},
        Arc, RwLock,
    },
    time::Instant,
};

use log::info;

use crate::{
    backend::{workspace::Workspace, BackendSettings, BackendStats, FullMachine},
    shared::{
        snapshot::log::{MessageType, StepMessage, StepStatus},
        BackendInfo, BackendStatus, Request, Response, StepSettings,
    },
};

use super::{extract_space_info, workspace::WorkspaceProperty};

struct WorkCommand {
    request: Request,
    send_to_server: SyncSender<Response>,
}

struct BackendWorker<M: FullMachine> {
    workspace: Workspace<M>,
    stats: Arc<RwLock<BackendStats>>,
    settings: BackendSettings,
    recv_from_server: Receiver<WorkCommand>,
}

impl<M: FullMachine> BackendWorker<M> {
    fn new(
        workspace: Workspace<M>,
        stats: Arc<RwLock<BackendStats>>,
        settings: BackendSettings,
        recv_from_server: Receiver<WorkCommand>,
    ) -> Self {
        Self {
            workspace,
            stats,
            settings,
            recv_from_server,
        }
    }

    fn run(mut self) {
        loop {
            let worker_request = match self.recv_from_server.recv() {
                Ok(ok) => ok,
                Err(_) => {
                    // all senders have disconnected
                    // end gracefully
                    break;
                }
            };

            // process the request
            self.process_request(worker_request);
        }
    }

    fn process_request(&mut self, work_command: WorkCommand) {
        enum AsynchronousRequest {
            Step(StepSettings),
        }

        // Perform synchronous processing first before sending the response.
        let asynchronous_request = match work_command.request {
            Request::InitialContent => None,
            Request::GetContent => None,
            Request::Query => {
                // do not waste time making a snapshot, just return the current stats
                let stats = self
                    .stats
                    .read()
                    .expect("Backend stats should not be poisoned");
                let backend_info = backend_info(false, &stats);
                let response = Response {
                    info: backend_info,
                    snapshot: None,
                };
                // ignore the error that occurs if the receiver does not exist anymore
                let _ = work_command.send_to_server.send(response);
                return;
            }
            Request::Cancel => {
                // there is nothing to cancel
                None
            }
            Request::Reset => {
                self.workspace.framework.reset();
                None
            }
            Request::Step(step_settings) => Some(AsynchronousRequest::Step(step_settings)),
            Request::AddProperty(prepared_property) => {
                self.workspace
                    .properties
                    .push(WorkspaceProperty::new(prepared_property));
                None
            }
            Request::RemoveProperty(root_property_index) => {
                self.workspace.properties.remove(root_property_index.0);
                None
            }
        };

        // Update the stats and send the response after synchronous processing.
        {
            let mut stats = self
                .stats
                .write()
                .expect("Backend stats should not be poisoned");
            // clear the cancellation flag before asynchronous processing
            stats.should_cancel = false;

            stats.space_info = extract_space_info(&self.workspace.framework);
            let worker_busy = asynchronous_request.is_some();
            let backend_info = backend_info(worker_busy, &stats);

            let response = Response {
                info: backend_info,
                snapshot: Some(self.workspace.generate_snapshot(&self.settings)),
            };

            // ignore the error that occurs if the receiver does not exist anymore
            let _ = work_command.send_to_server.send(response);
            // release the stats lock so the stats can be read by server threads
        }

        // Perform asynchronous processing.
        match asynchronous_request {
            Some(AsynchronousRequest::Step(step_settings)) => self.backend_step(step_settings),
            None => {}
        }
    }

    fn backend_step(&mut self, step_settings: StepSettings) {
        info!("Starting stepping.");

        // multi-step with possible cancellation between steps
        let start_instant = Instant::now();

        let mut num_refinements = 0;
        let mut cancelled = false;
        loop {
            // if the maximum number of refinements is given, stop stepping when it is reached
            if let Some(max_refinements) = step_settings.max_refinements {
                if num_refinements >= max_refinements {
                    break;
                }
            }

            // update the space info
            // if cancellation was requested, stop stepping
            if self.update_stats_check_cancel() {
                info!("Cancelling stepping.");
                cancelled = true;
                break;
            }

            // stop stepping when we are done
            // TODO: implement inherent assumption for the GUI
            if let ControlFlow::Break(_) = self
                .workspace
                .framework
                .step_verification(&step_settings.selected_property, false)
            {
                break;
            }

            num_refinements += 1;
        }
        let duration = start_instant.elapsed();

        self.workspace
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
        // we performed the work, update the stats
        self.update_stats_check_cancel();
        info!("Stepping done.");
    }

    fn update_stats_check_cancel(&self) -> bool {
        let mut stats_guard = self
            .stats
            .write()
            .expect("Backend stats should not be poisoned");
        stats_guard.space_info = extract_space_info(&self.workspace.framework);
        stats_guard.should_cancel
    }
}

pub struct BackendSync {
    stats: Arc<RwLock<BackendStats>>,
    send_to_worker: SyncSender<WorkCommand>,
}

impl BackendSync {}

impl BackendSync {
    pub fn new<M: FullMachine>(
        workspace: Workspace<M>,
        stats: BackendStats,
        settings: BackendSettings,
    ) -> BackendSync {
        let stats = Arc::new(RwLock::new(stats));

        // The server threads each try to send to the worker thread, which executes the requests sequentially.
        // Therefore, the bound 0 is used to make this a rendezvous channel.
        let (send_to_worker, recv_from_server) = sync_channel(0);

        // Spawn and detach the backend thread.
        let worker_stats = Arc::clone(&stats);
        std::thread::Builder::new()
            .name(String::from("backend worker"))
            .spawn(|| BackendWorker::new(workspace, worker_stats, settings, recv_from_server).run())
            .expect("Worker thread should be spawned");
        BackendSync {
            stats,
            send_to_worker,
        }
    }

    pub fn command(&self, request: Request) -> Response {
        // execute the worker
        let is_cancel = matches!(request, Request::Cancel);

        match self.try_execute_worker(request) {
            Ok(ok) => ok,
            Err(_) => {
                // the worker is currently busy
                let info = if is_cancel {
                    // try to cancel the computation
                    let mut stats = self.lock_stats_write();
                    stats.should_cancel = true;
                    backend_info(true, &stats)
                } else {
                    // just read the stats and construct the response
                    let stats = self.lock_stats_read();
                    backend_info(true, &stats)
                };
                Response {
                    info,
                    snapshot: None,
                }
            }
        }
    }

    fn try_execute_worker(&self, request: Request) -> Result<Response, ()> {
        // The worker thread sends an one-shot message to the originator server thread, do not block the worker.
        // As Rust does not provide a one-shot channel in std, just use a synchronous channel with bound 1.
        let (send_to_server, recv_from_worker) = sync_channel(1);

        let is_initial_content_request = matches!(request, Request::InitialContent);

        let worker_request = WorkCommand {
            request,
            send_to_server,
        };

        // The initial content request must have a snapshot inside its response. Ensure it does by blocking.
        if is_initial_content_request {
            if self.send_to_worker.send(worker_request).is_err() {
                panic!("Backend worker should not disconnect (service sending)");
            }
        } else {
            match self.send_to_worker.try_send(worker_request) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {
                    // the worker is busy
                    return Err(());
                }
                Err(TrySendError::Disconnected(_)) => {
                    panic!("Backend worker should not disconnect (service sending)");
                }
            };
        }

        let response = recv_from_worker
            .recv()
            .expect("Backend worker should not disconnect (service receiving)");

        Ok(response)
    }

    fn lock_stats_read(&self) -> std::sync::RwLockReadGuard<'_, BackendStats> {
        self.stats
            .read()
            .expect("Backend stats should not be poisoned")
    }

    fn lock_stats_write(&self) -> std::sync::RwLockWriteGuard<'_, BackendStats> {
        self.stats
            .write()
            .expect("Backend stats should not be poisoned")
    }
}

fn backend_info(worker_busy: bool, stats: &BackendStats) -> BackendInfo {
    let status = if worker_busy {
        if stats.should_cancel {
            BackendStatus::Cancelling
        } else {
            BackendStatus::Running
        }
    } else {
        BackendStatus::Waiting
    };
    BackendInfo {
        status,
        space_info: stats.space_info.clone(),
    }
}
