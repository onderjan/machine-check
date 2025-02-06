use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use super::view::Content;

pub async fn fetch() -> Content {
    let result = inner_fetch().await;
    let json = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };
    serde_wasm_bindgen::from_value(json).expect("Content should be convertible from JSON")
}

pub async fn inner_fetch() -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = "/api/content";

    let request = Request::new_with_str(url)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json()?).await?;

    Ok(json)
}
