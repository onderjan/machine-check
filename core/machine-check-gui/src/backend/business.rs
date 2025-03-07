use std::{borrow::Cow, sync::RwLock};

use http::{header::CONTENT_TYPE, Method};
use include_dir::{include_dir, Dir};
use log::{debug, error};
use machine_check_exec::Framework;
use mck::concr::FullMachine;

use crate::frontend::interaction::Request;

mod api;

const CONTENT_DIR: Dir = include_dir!("content");

pub struct Business<M: FullMachine> {
    framework: Framework<M>,
    exec_name: String,
}

impl<M: FullMachine> Business<M> {
    pub fn new(framework: Framework<M>, exec_name: String) -> Self {
        Business {
            framework,
            exec_name,
        }
    }

    pub fn get_http_response(
        business: &RwLock<Self>,
        request: http::Request<Vec<u8>>,
    ) -> http::Response<Cow<'static, [u8]>> {
        Business::get_response(business, request).unwrap_or_else(|err| {
            error!("{}", err);
            let response = http::Response::builder()
                .header(CONTENT_TYPE, "text/plain")
                .status(500)
                .body(Cow::Borrowed("Internal Server Error".as_bytes()))
                .expect("Internal server error response should be constructable");
            response
        })
    }

    fn get_response(
        business: &RwLock<Self>,
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

            Self::get_api_response(business, request)
        } else {
            // not an API call, return content
            if method != Method::GET {
                return Err(anyhow::anyhow!("Expected method GET: {}", path).into());
            }

            Self::get_content_response(path)
        }
    }

    fn get_api_response(
        business: &RwLock<Self>,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
        let request: Request = rmp_serde::from_slice(request.body())?;

        let business = &mut business.write().expect("Lock should not be poisoned");
        match request {
            Request::GetContent => {}
            Request::Step(step_settings) => {
                business
                    .framework
                    .step_verification(step_settings.num_steps);
            }
        }

        // read the current framework state
        let content = api::api_response(business)?;

        http::Response::builder()
            .header(CONTENT_TYPE, "application/vnd.msgpack")
            .body(content)
            .map_err(Into::into)
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
}
