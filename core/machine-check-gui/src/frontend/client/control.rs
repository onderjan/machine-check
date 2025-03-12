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
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::Cors);

    let body = rmp_serde::to_vec(&request).expect("Action should be serializable");
    let body = Uint8Array::from(body.as_slice());

    opts.set_body(&body);

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
