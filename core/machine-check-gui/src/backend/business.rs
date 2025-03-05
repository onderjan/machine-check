use std::{borrow::Cow, sync::RwLock};

use http::{header::CONTENT_TYPE, Method, Request, Response};
use include_dir::{include_dir, Dir};
use log::{debug, error};
use machine_check_exec::Framework;
use mck::concr::FullMachine;

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
        request: Request<Vec<u8>>,
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

        let (content, content_type) = match path.strip_prefix("api/") {
            Some(api_path) => {
                let content = match api_path {
                    "step_verification" => {
                        if method != Method::POST {
                            return Err(anyhow::anyhow!(
                                "Verification step API command method must be POST"
                            )
                            .into());
                        }

                        // step verification
                        let business = &mut business.write().expect("Lock should not be poisoned");
                        business.framework.step_verification();
                        api::api_response(business)?
                    }
                    "reset" => {
                        if method != Method::POST {
                            return Err(anyhow::anyhow!(
                                "Reset API command method must be POST, is {}",
                                method
                            )
                            .into());
                        }
                        todo!("Reset the verification");
                    }
                    "content" => {
                        if method != Method::GET {
                            return Err(anyhow::anyhow!(
                                "Content API command method must be GET, is {}",
                                method
                            )
                            .into());
                        }
                        // read the current framework state
                        let business = &business.read().expect("Lock should not be poisoned");
                        api::api_response(business)?
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

                let content = match CONTENT_DIR.get_file(path) {
                    Some(file) => file.contents(),
                    None => return Err(anyhow::anyhow!("Not found: {}", path).into()),
                };

                let content_type = Cow::Owned(
                    mime_guess::from_path(path)
                        .first()
                        .expect("Content should have known content type")
                        .to_string(),
                );
                (Cow::Borrowed(content), content_type)
            }
        };

        Response::builder()
            .header(CONTENT_TYPE, content_type.as_ref())
            .body(content)
            .map_err(Into::into)
    }
}
