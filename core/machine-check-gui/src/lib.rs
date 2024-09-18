use std::borrow::Cow;

use gui::Gui;
use http::{header::CONTENT_TYPE, Method, Request, Response};
use log::{debug, error};
use machine_check_common::ExecError;

mod gui;

pub fn run() -> Result<(), ExecError> {
    // initialise the GUI
    let gui = match Gui::new(get_http_response) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Cannot create GUI: {}", err);
            return Err(ExecError::GuiError(err.to_string()));
        }
    };
    // run the GUI, never returns
    gui.run()
}

const INDEX_HTML: &str = include_str!("../content/index.html");
const SCRIPT_JS: &str = include_str!("../content/script.js");

fn get_response(
    request: Request<Vec<u8>>,
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
                return Err(
                    anyhow::anyhow!("Path not empty or starting with slash: {}", uri_path).into(),
                );
            }
        }
    };

    // if the stripped path is empty, serve index.html
    let path = if path.is_empty() { "index.html" } else { path };

    let (content, content_type) = match path.strip_prefix("api/") {
        Some(api_path) => {
            let content = match api_path {
                "refine" => {
                    if method != Method::POST {
                        return Err(
                            anyhow::anyhow!("Refine API command method must be POST").into()
                        );
                    }
                    // TODO
                    Cow::Borrowed("{}".as_bytes())
                }
                "reset" => {
                    if method != Method::POST {
                        return Err(anyhow::anyhow!("Reset API command method must be POST").into());
                    }
                    // TODO
                    Cow::Borrowed("{}".as_bytes())
                }
                "content" => {
                    if method != Method::GET {
                        return Err(
                            anyhow::anyhow!("Content API command method must be GET").into()
                        );
                    }
                    // TODO
                    Cow::Borrowed("{\"space\": {\"states\": {}, \"edges\": {}}}".as_bytes())
                }
                _ => return Err(anyhow::anyhow!("Unknown API command: {}", path).into()),
            };
            let content_type = Cow::Borrowed("text/json");
            (content, content_type)
        }
        None => {
            if method != Method::GET {
                return Err(anyhow::anyhow!("Expected method GET: {}", path).into());
            }
            let content = match path {
                "index.html" => Cow::Borrowed(INDEX_HTML.as_bytes()),
                "script.js" => Cow::Borrowed(SCRIPT_JS.as_bytes()),
                _ => return Err(anyhow::anyhow!("Not found: {}", path).into()),
            };

            let content_type = Cow::Owned(
                mime_guess::from_path(path)
                    .first()
                    .expect("Content should have known content type")
                    .to_string(),
            );
            (content, content_type)
        }
    };

    Response::builder()
        .header(CONTENT_TYPE, content_type.as_ref())
        .body(content)
        .map_err(Into::into)
}

fn get_http_response(request: Request<Vec<u8>>) -> http::Response<Cow<'static, [u8]>> {
    get_response(request).unwrap_or_else(|err| {
        error!("{}", err);
        let response = http::Response::builder()
            .header(CONTENT_TYPE, "text/plain")
            .status(500)
            .body(Cow::Borrowed("Internal Server Error".as_bytes()))
            .expect("Internal server error response should be constructable");
        response
    })
}
