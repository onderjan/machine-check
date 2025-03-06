mod fields;
mod mouse;
mod render;
mod view;

use std::cell::RefCell;

use machine_check_exec::NodeId;
use view::View;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{ArrayBuffer, Uint8Array},
    Request, RequestInit, RequestMode, Response,
};

use super::{content::Content, util::PixelPoint};

pub enum Action {
    GetContent,
    Step,
}

#[derive(Debug)]
pub struct PointOfView {
    pub view_offset: PixelPoint,
    mouse_current_coords: Option<PixelPoint>,
    mouse_down_coords: Option<PixelPoint>,
    selected_node_id: Option<NodeId>,
}

impl PointOfView {
    fn new() -> Self {
        PointOfView {
            view_offset: PixelPoint { x: 0, y: 0 },
            mouse_current_coords: None,
            mouse_down_coords: None,
            selected_node_id: None,
        }
    }

    fn view_offset(&self) -> PixelPoint {
        let mut result = self.view_offset;
        if let (Some(mouse_down_px), Some(mouse_current_px)) =
            (self.mouse_down_coords, self.mouse_current_coords)
        {
            let mouse_offset = mouse_current_px - mouse_down_px;
            result -= mouse_offset;
        }
        result
    }
}

thread_local! {
    static VIEW: RefCell<Option<View>> = const { RefCell::new(None) };
    static POINT_OF_VIEW: RefCell<PointOfView> = RefCell::new(PointOfView::new());
}

pub async fn update(action: Action, resize: bool) {
    execute_action(action).await;
    display_current(resize);
}

pub async fn display(resize: bool) {
    let should_execute = VIEW.with(|view| view.borrow().is_none());
    if should_execute {
        execute_action(Action::GetContent).await;
    }

    display_current(resize);
}

fn display_current(resize: bool) {
    VIEW.with(|view| {
        let view_guard = view.borrow();
        let Some(ref view) = *view_guard else {
            panic!("View should be loaded");
        };
        POINT_OF_VIEW.with(|point_of_view| {
            let point_of_view = point_of_view.borrow();
            fields::display(view, &point_of_view);
            render::render(view, &point_of_view, resize);
        });
    });
}

async fn execute_action(action: Action) {
    let result = call_backend(action).await;
    let response = match result {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    };

    console_log!(&format!("Response byte length: {}", response.byte_length()));

    let response = Uint8Array::new(&response);
    let response = response.to_vec();
    let content: Content =
        rmp_serde::from_slice(&response).expect("Content should be convertible from Messagepack");

    let view = View::new(content);
    console_log!(&format!("View: {:?}", view));
    VIEW.replace(Some(view));
}

pub async fn call_backend(action: Action) -> Result<ArrayBuffer, JsValue> {
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

    let response: Response = resp_value.dyn_into().unwrap();

    console_log!(&format!("WebSys Response: {:?}", response));

    let response: ArrayBuffer = JsFuture::from(response.array_buffer()?).await?.into();

    Ok(response)
}

pub async fn on_mouse(mouse: super::MouseEvent, event: web_sys::Event) {
    let render = VIEW.with_borrow(|view| {
        // only process mouse events if we have a view
        if let Some(view) = view {
            POINT_OF_VIEW
                .with_borrow_mut(|point_of_view| mouse::on_mouse(view, point_of_view, mouse, event))
        } else {
            false
        }
    });

    if render {
        display_current(true);
    }
}
