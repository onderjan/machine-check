use gui::Gui;
use http::{header::CONTENT_TYPE, Request, Response};
use log::{debug, error};

mod gui;

fn main() -> wry::Result<()> {
    // initialise the logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // initialise the GUI
    let gui = match Gui::new(get_http_response) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Cannot create GUI: {}", err);
            return Err(err);
        }
    };
    // run the GUI, never returns
    gui.run()
}

const INDEX_HTML: &str = include_str!("../content/index.html");

fn get_response(
    request: Request<Vec<u8>>,
) -> Result<http::Response<std::borrow::Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
    // read URI path
    let uri_path = request.uri().path();
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

    let content = match path {
        "index.html" => std::borrow::Cow::Borrowed(INDEX_HTML.as_bytes()),
        _ => return Err(anyhow::anyhow!("Not found: {}", path).into()),
    };
    let content_type = mime_guess::from_path(path).first().unwrap().to_string();

    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .body(content)
        .map_err(Into::into)
}

fn get_http_response(request: Request<Vec<u8>>) -> http::Response<std::borrow::Cow<'static, [u8]>> {
    get_response(request).unwrap_or_else(|err| {
        error!("{}", err);
        let response = http::Response::builder()
            .header(CONTENT_TYPE, "text/plain")
            .status(500)
            .body(std::borrow::Cow::Borrowed(
                "Internal Server Error".as_bytes(),
            ))
            .expect("Internal server error response should be constructable");
        response
    })
}
