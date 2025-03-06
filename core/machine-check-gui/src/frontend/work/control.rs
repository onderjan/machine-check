use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{ArrayBuffer, Uint8Array};

use crate::frontend::{
    interaction::{Request, Response},
    snapshot::Snapshot,
};

pub async fn command(request: Request) -> Snapshot {
    let result = call_backend(request).await;
    let response = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };

    console_log!(&format!("Response byte length: {}", response.byte_length()));

    let response = Uint8Array::new(&response);
    let response = response.to_vec();
    let response: Response =
        rmp_serde::from_slice(&response).expect("Content should be convertible from Messagepack");

    response.snapshot
}

pub async fn call_backend(request: Request) -> Result<ArrayBuffer, JsValue> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::Cors);

    let body = rmp_serde::to_vec(&request).expect("Action should be serializable");
    let body = Uint8Array::from(body.as_slice());

    opts.set_body(&body);

    let request = web_sys::Request::new_with_str_and_init("/api", &opts)?;

    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.dyn_into().unwrap();
    let response: ArrayBuffer = JsFuture::from(response.array_buffer()?).await?.into();

    Ok(response)
}
