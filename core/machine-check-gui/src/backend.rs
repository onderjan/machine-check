use std::{
    borrow::Cow,
    ffi::OsStr,
    path::Path,
    sync::{Arc, Mutex},
};

use http::{header::CONTENT_TYPE, Method};
use include_dir::{include_dir, Dir};
use log::{debug, error};
use machine_check_common::ExecError;
use machine_check_exec::{Framework, Proposition, Strategy};
use mck::concr::FullMachine;
use window::Window;
use workspace::Workspace;
use wry::WebViewId;

use crate::shared::Request;

mod api;
mod window;
mod workspace;

const FAVICON_ICO: &[u8] = include_bytes!("../content/favicon.ico");

pub fn run<M: FullMachine>(
    system: M,
    property: Option<Proposition>,
    strategy: Strategy,
) -> Result<(), ExecError> {
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
    // create the backend
    let backend = Backend::new(
        Workspace::<M>::new(Framework::new(abstract_system, &strategy), property),
        exec_name.clone(),
    );
    let response_fn = move |_web_view_id: WebViewId, request: http::Request<Vec<u8>>| {
        backend.get_http_response(request)
    };

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

struct Backend<M: FullMachine> {
    workspace: Arc<Mutex<Workspace<M>>>,
    stats: Arc<Mutex<BackendStats>>,
    settings: BackendSettings,
}

struct BackendStats {
    running: bool,
    should_cancel: bool,
}

impl BackendStats {
    fn new() -> Self {
        Self {
            running: false,
            should_cancel: false,
        }
    }
}

struct BackendSettings {
    exec_name: String,
}

const CONTENT_DIR: Dir = include_dir!("content");

impl<M: FullMachine> Backend<M> {
    pub fn new(workspace: Workspace<M>, exec_name: String) -> Self {
        Self {
            workspace: Arc::new(Mutex::new(workspace)),
            stats: Arc::new(Mutex::new(BackendStats::new())),
            settings: BackendSettings { exec_name },
        }
    }

    fn get_http_response(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> http::Response<Cow<'static, [u8]>> {
        // handle errors by printing them and sending 500
        self.get_http_response_or_error(request)
            .unwrap_or_else(|err| {
                error!("{}", err);
                let response = http::Response::builder()
                    .header(CONTENT_TYPE, "text/plain")
                    .status(500)
                    .body(Cow::Borrowed("Internal Server Error".as_bytes()))
                    .expect("Internal server error response should be constructable");
                response
            })
    }

    fn get_http_response_or_error(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
        // read URI path
        let uri_path = request.uri().path();
        let method = request.method();
        debug!("Serving: {}", uri_path);

        // strip the leading slash
        // also accept empty path
        let path = match uri_path.strip_prefix('/') {
            Some(path) => path,
            None => {
                if uri_path.is_empty() {
                    ""
                } else {
                    return Err(anyhow::anyhow!(
                        "Path not empty or starting with slash: {}",
                        uri_path
                    )
                    .into());
                }
            }
        };

        // if the stripped path is empty, serve index.html
        let path = if path.is_empty() { "index.html" } else { path };

        if path == "api" {
            // API call
            if method != Method::POST {
                return Err(anyhow::anyhow!("API method must be POST").into());
            }

            self.get_api_response(request)
        } else {
            // not an API call, return content
            if method != Method::GET {
                return Err(anyhow::anyhow!("Expected method GET: {}", path).into());
            }

            Self::get_content_response(path)
        }
    }

    fn get_content_response(
        path: &str,
    ) -> Result<http::Response<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
        let content = match CONTENT_DIR.get_file(path) {
            Some(file) => file.contents(),
            None => return Err(anyhow::anyhow!("Not found: {}", path).into()),
        };

        let content_type: Cow<str> = Cow::Owned(
            mime_guess::from_path(path)
                .first()
                .expect("Content should have known content type")
                .to_string(),
        );

        http::Response::builder()
            .header(CONTENT_TYPE, content_type.as_ref())
            .body(Cow::Borrowed(content))
            .map_err(Into::into)
    }

    fn get_api_response(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
        let request: Request = rmp_serde::from_slice(request.body())?;

        // read the current framework state
        let response = api::command(self, request);

        let content_msgpack = rmp_serde::to_vec(&response)?;
        http::Response::builder()
            .header(CONTENT_TYPE, "application/vnd.msgpack")
            .body(Cow::Owned(content_msgpack))
            .map_err(Into::into)
    }
}
