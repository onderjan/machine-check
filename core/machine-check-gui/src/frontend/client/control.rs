use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{ArrayBuffer, Uint8Array};

use crate::{
    frontend::util::web_idl::window,
    shared::{BackendStatus, Request, Response},
};

mod properties;
mod verification;

pub fn init() {
    verification::init();
    properties::init();
}

pub async fn call_backend(request: Request) -> Response {
    let result = call_backend_fetch(request).await;
    let response = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };

    console_log!("Response byte length: {}", response.byte_length());

    let response = Uint8Array::new(&response);
    let response = response.to_vec();
    let response: Response =
        rmp_serde::from_slice(&response).expect("Content should be convertible from Messagepack");

    console_log!("Response: {:?}", response);

    response
}

pub async fn call_backend_fetch(request: Request) -> Result<ArrayBuffer, JsValue> {
    // as posting the request content in the body seems buggy (we can encounter
    // an empty body instead), we instead send the request body is in the header
    // X-Body, encoded into a hex
    let body_msgpack = rmp_serde::to_vec(&request).expect("Action should be serializable");
    let body_hex = hex::encode(body_msgpack);

    let headers = web_sys::Headers::new()?;
    headers.append("X-Body", &body_hex)?;
    let opts = web_sys::RequestInit::new();
    opts.set_headers(&headers);
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init("/api", &opts)?;

    let response = JsFuture::from(window().fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.dyn_into().unwrap();
    let response: ArrayBuffer = JsFuture::from(response.array_buffer()?).await?.into();

    Ok(response)
}

pub fn display_backend_status(backend_status: &BackendStatus) {
    verification::display_backend_status(backend_status);
    properties::display_backend_status(backend_status);
}
