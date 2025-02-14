mod mouse;
mod render;
mod view;

use std::cell::RefCell;

use view::View;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::Array, Request, RequestInit, RequestMode, Response};

use super::content::Content;

pub enum Action {
    GetContent,
    Step,
}

#[derive(Debug, Default)]
pub struct PointOfView {
    pub translation_px: (f64, f64),
    mouse_down_px: Option<(i32, i32)>,
    mouse_current_px: Option<(i32, i32)>,
}

impl PointOfView {
    fn translation(&self) -> (f64, f64) {
        let mut x = self.translation_px.0;
        let mut y = self.translation_px.1;
        if let (Some(mouse_down_px), Some(mouse_current_px)) =
            (self.mouse_down_px, self.mouse_current_px)
        {
            let offset_x = mouse_current_px.0 - mouse_down_px.0;
            let offset_y = mouse_current_px.1 - mouse_down_px.1;
            x += offset_x as f64;
            y += offset_y as f64;
        }
        (x, y)
    }
}

thread_local! {
    static VIEW: RefCell<Option<View>> = const { RefCell::new(None) };
    static POINT_OF_VIEW: RefCell<PointOfView> = RefCell::default();
}

pub async fn update(action: Action, resize: bool) {
    execute_action(action).await;
    render_current(resize);
}

pub async fn render(resize: bool) {
    let should_execute = VIEW.with(|view| view.borrow().is_none());
    if should_execute {
        execute_action(Action::GetContent).await;
    }

    render_current(resize);
}

fn render_if_available(resize: bool) {
    VIEW.with(|view| {
        let view_guard = view.borrow();
        if let Some(ref view) = *view_guard {
            POINT_OF_VIEW.with(|point_of_view| {
                render::render(view, &point_of_view.borrow(), resize);
            });
        }
    });
}

fn render_current(resize: bool) {
    VIEW.with(|view| {
        let view_guard = view.borrow();
        let Some(ref view) = *view_guard else {
            panic!("View should be loaded");
        };
        POINT_OF_VIEW.with(|point_of_view| {
            render::render(view, &point_of_view.borrow(), resize);
        });
    });
}

async fn execute_action(action: Action) {
    let result = call_backend(action).await;
    let json = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };
    let content: Content =
        serde_wasm_bindgen::from_value(json).expect("Content should be convertible from JSON");

    let view = View::new(content);
    let cons = Array::new_with_length(1);
    cons.set(0, JsValue::from_str(&format!("{:?}", view)));
    web_sys::console::log(&cons);

    VIEW.replace(Some(view));
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

pub async fn on_mouse(mouse: super::MouseEvent, event: web_sys::Event) {
    let render =
        POINT_OF_VIEW.with_borrow_mut(|point_of_view| mouse::on_mouse(point_of_view, mouse, event));

    if render {
        render_if_available(true);
    }
}
