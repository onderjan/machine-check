mod fields;
mod mouse;
mod render;
mod view;

use std::cell::RefCell;

use machine_check_exec::NodeId;
use view::View;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{ArrayBuffer, Uint8Array};

use crate::frontend::interaction::Response;

use super::{interaction::Request, util::PixelPoint};

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

pub async fn update(request: Request, resize: bool) {
    execute_request(request).await;
    display_current(resize);
}

pub async fn display(resize: bool) {
    let should_execute = VIEW.with(|view| view.borrow().is_none());
    if should_execute {
        execute_request(Request::GetContent).await;
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

async fn execute_request(request: Request) {
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

    let view = View::new(response.snapshot);
    console_log!(&format!("View: {:?}", view));
    VIEW.replace(Some(view));
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
