use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use gui::Gui;
use http::{header::CONTENT_TYPE, Method, Request, Response};
use log::{debug, error};
use machine_check_common::ExecError;
use machine_check_exec::{Framework, NodeId};
use mck::concr::FullMachine;
use serde::{Deserialize, Serialize};

mod gui;

pub fn run<M: FullMachine>(system: M) -> Result<(), ExecError> {
    let abstract_system = <M::Abstr as mck::abstr::Abstr<M>>::from_concrete(system);
    // create the business logic
    let business = RwLock::new(Business::<M> {
        framework: Framework::new(
            abstract_system,
            machine_check_exec::VerificationType::Inherent,
            &machine_check_exec::Strategy {
                naive_inputs: false,
                use_decay: false,
            },
        ),
    });

    //let business = Arc::new(business);

    let response_fn = move |request| Business::get_http_response(&business, request);

    // initialise the GUI
    let gui = match Gui::new(response_fn) {
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
const RENDER_JS: &str = include_str!("../content/render.js");
const UPDATE_JS: &str = include_str!("../content/update.js");
const STYLE_CSS: &str = include_str!("../content/style.css");
const FAVICON_ICO: &[u8] = include_bytes!("../content/favicon.ico");

struct Business<M: FullMachine> {
    framework: Framework<M>,
}

impl<M: FullMachine> Business<M> {
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
                        let framework = &mut business
                            .write()
                            .expect("Lock should not be poisoned")
                            .framework;
                        framework.step_verification();
                        framework_response(framework)?
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
                        let framework = &business
                            .read()
                            .expect("Lock should not be poisoned")
                            .framework;
                        framework_response(framework)?
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
                    "render.js" => Cow::Borrowed(RENDER_JS.as_bytes()),
                    "update.js" => Cow::Borrowed(UPDATE_JS.as_bytes()),
                    "style.css" => Cow::Borrowed(STYLE_CSS.as_bytes()),
                    "favicon.ico" => Cow::Borrowed(FAVICON_ICO),
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

    fn get_http_response(
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
}

#[derive(Serialize, Deserialize)]
struct ThreeValuedBool {
    zero: bool,
    one: bool,
}

#[derive(Serialize, Deserialize)]
struct Node {
    incoming: BTreeSet<String>,
    outgoing: BTreeSet<String>,
    panic: Option<ThreeValuedBool>,
    fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct StateSpace {
    // represent the IDs by strings for now
    nodes: BTreeMap<String, Node>,
}

#[derive(Serialize, Deserialize)]
struct StateInfo {
    field_names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    state_space: StateSpace,
    state_info: StateInfo,
}

fn framework_response<M: FullMachine>(
    framework: &Framework<M>,
) -> Result<Cow<'static, [u8]>, Box<dyn std::error::Error>> {
    let state_field_names: Vec<String> =
        <<M::Abstr as mck::abstr::Machine<M>>::State as mck::abstr::Manipulatable>::field_names()
            .into_iter()
            .map(String::from)
            .collect();

    let state_info = StateInfo {
        field_names: state_field_names.clone(),
    };

    let state_map = framework.space().state_map();
    let node_graph = framework.space().node_graph();

    let node_iter = std::iter::once((NodeId::START, None)).chain(
        state_map
            .iter()
            .map(|(state_id, state)| ((*state_id).into(), Some(state))),
    );

    let mut nodes = BTreeMap::new();
    for (node_id, state) in node_iter {
        let incoming = node_graph
            .neighbors_directed(node_id, petgraph::Direction::Incoming)
            .map(|incoming_id| incoming_id.to_string())
            .collect();
        let outgoing = node_graph
            .neighbors_directed(node_id, petgraph::Direction::Outgoing)
            .map(|outgoing_id| outgoing_id.to_string())
            .collect();
        let (fields, panic) = if let Some(state) = state {
            let panic_result = &state.0;
            let panic = ThreeValuedBool {
                zero: panic_result.panic.umin().is_zero(),
                one: panic_result.panic.umax().is_nonzero(),
            };
            let mut fields = BTreeMap::new();
            for field_name in state_field_names.iter() {
                let field_get = mck::abstr::Manipulatable::get(&panic_result.result, field_name)
                    .expect("Field name should correspond to a field");
                let description = field_get.json_description();

                fields.insert(field_name.clone(), description);
            }
            (fields, Some(panic))
        } else {
            (BTreeMap::new(), None)
        };

        let node_info = Node {
            incoming,
            outgoing,
            panic,
            fields,
        };
        nodes.insert(node_id.to_string(), node_info);
    }

    let state_space = StateSpace { nodes };

    let content = Content {
        state_space,
        state_info,
    };

    let content_json = serde_json::to_string(&content)?;

    Ok(Cow::Owned(content_json.into_bytes()))
}
