use std::{borrow::Cow, ffi::OsStr, path::Path};

use http::{header::CONTENT_TYPE, Method};
use include_dir::{include_dir, Dir};
use log::{debug, error};
use machine_check_common::ExecError;
use machine_check_exec::{Framework, Property, Strategy};
use mck::concr::FullMachine;
use sync::BackendSync;
use window::Window;
use workspace::Workspace;
use wry::WebViewId;

use crate::shared::{BackendSpaceInfo, Request};

mod sync;
mod window;
mod workspace;

const FAVICON_ICO: &[u8] = include_bytes!("../content/favicon.ico");

pub fn run<M: FullMachine>(
    system: M,
    property: Option<Property>,
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

struct Backend {
    sync: BackendSync,
    /*workspace: Arc<Mutex<Workspace<M>>>,
    stats: Arc<Mutex<BackendStats>>,
    settings: BackendSettings,*/
}

struct BackendStats {
    should_cancel: bool,
    space_info: BackendSpaceInfo,
}

impl BackendStats {
    fn new<M: FullMachine>(framework: &Framework<M>) -> Self {
        Self {
            should_cancel: false,
            space_info: extract_space_info(framework),
        }
    }
}

fn extract_space_info<M: FullMachine>(framework: &Framework<M>) -> BackendSpaceInfo {
    let num_states = framework.info().num_final_states;
    let num_transitions = framework.info().num_final_transitions;
    BackendSpaceInfo {
        num_states,
        num_transitions,
    }
}

struct BackendSettings {
    exec_name: String,
}

const CONTENT_DIR: Dir = include_dir!("content");

impl Backend {
    pub fn new<M: FullMachine>(workspace: Workspace<M>, exec_name: String) -> Self {
        let stats = BackendStats::new(&workspace.framework);
        let settings = BackendSettings { exec_name };
        let sync = BackendSync::new(workspace, stats, settings);
        Self {
            sync, /*workspace: Arc::new(Mutex::new(workspace)),
                  stats: Arc::new(Mutex::new(stats)),
                  settings: BackendSettings { exec_name },*/
        }
    }

    fn get_http_response(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> http::Response<Cow<'static, [u8]>> {
        // handle errors by printing them and sending 500
        self.get_http_response_or_error(request)
            .unwrap_or_else(|err| {
                error!("Cannot produce a response to frontend: {}", err);
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
        // as posting the request content in the body seems buggy (we can encounter
        // an empty body instead), the request body is instead sent in the header
        // X-Body, encoded into a hex
        let x_body = request
            .headers()
            .get("X-Body")
            .ok_or(anyhow::anyhow!("Request has no X-Body header"))?;
        let x_body = x_body
            .to_str()
            .map_err(|_| anyhow::anyhow!("Request X-Body header is not ASCII"))?;
        let decoded_body = hex::decode(x_body).map_err(|err| {
            anyhow::anyhow!("Request X-Body header does not contain hex: {}", err)
        })?;
        let request: Request = rmp_serde::from_slice(&decoded_body).map_err(|err| {
            anyhow::anyhow!(
                "Request X-Body header does not contain valid MessagePack data: {}",
                err
            )
        })?;

        // create the response
        let response = self.sync.command(request);

        // msgpack the response
        let content_msgpack = rmp_serde::to_vec(&response)?;
        http::Response::builder()
            .header(CONTENT_TYPE, "application/vnd.msgpack")
            .body(Cow::Owned(content_msgpack))
            .map_err(Into::into)
    }
}
