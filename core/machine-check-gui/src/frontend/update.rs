mod render;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::Array, Request, RequestInit, RequestMode, Response};

use super::view::Content;

pub enum Action {
    GetContent,
    Step,
}

pub async fn update(action: Action) {
    let result = call_backend(action).await;
    let json = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };
    let content: Content =
        serde_wasm_bindgen::from_value(json).expect("Content should be convertible from JSON");

    let cons = Array::new_with_length(1);
    cons.set(0, JsValue::from_str(&format!("{:?}", content)));
    web_sys::console::log(&cons);

    render::render(&content);
}

pub async fn call_backend(action: Action) -> Result<JsValue, JsValue> {
    let (method, url_action) = match action {
        Action::GetContent => ("GET", "content"),
        Action::Step => ("POST", "step_verification"),
    };

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);

    let url = format!("/api/{}", url_action);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json()?).await?;

    Ok(json)
}
