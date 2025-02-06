use wasm_bindgen::{JsCast, JsValue};
use web_sys::{js_sys, CanvasRenderingContext2d, Element, HtmlCanvasElement};

use crate::frontend::view::Content;

#[derive(Clone, Copy)]
struct Tile {
    x: u64,
    y: u64,
}

pub fn render(content: &Content) {
    LOCAL.with(|local| {
        Renderer { content, local }.render();
    });
}

const RAW_TILE_SIZE: f64 = 46.;
const RAW_NODE_SIZE: f64 = 30.;

struct Renderer<'a> {
    content: &'a Content,
    local: &'a Local,
}

impl Renderer<'_> {
    fn render(&self) {
        // TODO: only do this when resizing canvas
        self.fix_resized_canvas();

        // TODO: render nodes properly
        for (index, (node_id, _node)) in self.content.state_space.nodes.iter().enumerate() {
            self.render_node(
                Tile {
                    x: index as u64,
                    y: 0,
                },
                node_id,
            );
        }
    }

    fn tile_size(&self) -> f64 {
        RAW_TILE_SIZE * self.local.pixel_ratio
    }

    fn node_size(&self) -> f64 {
        RAW_NODE_SIZE * self.local.pixel_ratio
    }

    fn render_node(&self, tile: Tile, node_id: &str) {
        let context = &self.local.main_context;

        context.begin_path();

        let tile_size = self.tile_size();
        let node_size = self.node_size();

        let node_start_x = tile.x as f64 * tile_size + (tile_size - node_size) / 2.;
        let node_start_y = tile.y as f64 * tile_size + (tile_size - node_size) / 2.;

        context.stroke_rect(node_start_x, node_start_y, node_size, node_size);

        context
            .fill_text(
                node_id,
                node_start_x + node_size / 2.,
                node_start_y + node_size / 2.,
            )
            .unwrap();
    }

    fn fix_resized_canvas(&self) {
        // fix for device pixel ratio
        let pixel_ratio = self.local.pixel_ratio;
        let main_area_rect = self.local.main_area.get_bounding_client_rect();
        let width = main_area_rect.width();
        let height = main_area_rect.height();
        self.local
            .main_canvas
            .set_width((width * pixel_ratio) as u32);
        self.local
            .main_canvas
            .set_height((height * pixel_ratio) as u32);

        let canvas_style = self.local.main_canvas.style();
        canvas_style
            .set_property("width", &format!("{}px", width))
            .unwrap();
        canvas_style
            .set_property("height", &format!("{}px", height))
            .unwrap();

        // set font size
        let font_size = 12. * pixel_ratio;
        self.local
            .main_context
            .set_font(&format!("{}px sans-serif", font_size));
        self.local.main_context.set_text_align("center");
        self.local.main_context.set_text_baseline("middle");

        // make sure we stroke true pixels
        self.local.main_context.reset_transform().unwrap();
        self.local.main_context.translate(0.5, 0.5).unwrap();
    }
}

struct Local {
    main_area: Element,
    main_canvas: HtmlCanvasElement,
    main_context: CanvasRenderingContext2d,
    pixel_ratio: f64,
}

impl Local {
    fn new() -> Local {
        let window = web_sys::window().expect("HTML Window should exist");
        let document = window.document().expect("HTML document should exist");
        let main_area = document
            .get_element_by_id("main_area")
            .expect("Main area should exist");
        let main_canvas = document
            .get_element_by_id("main_canvas")
            .expect("Main canvas should exist");
        let main_canvas: HtmlCanvasElement = main_canvas
            .dyn_into()
            .expect("Main canvas should be a Canvas element");
        let main_context: CanvasRenderingContext2d = main_canvas
            .get_context("2d")
            .expect("Main canvas 2D context should be obtainable without an error")
            .expect("Main canvas should have a 2D context")
            .dyn_into()
            .expect("Main canvas 2D rendering context should be castable");
        let pixel_ratio = window.device_pixel_ratio();

        let cons = js_sys::Array::new_with_length(1);
        cons.set(0, JsValue::from_str("Pixel ratio"));
        cons.set(1, JsValue::from_f64(pixel_ratio));
        web_sys::console::log(&cons);

        Local {
            main_area,
            main_canvas,
            main_context,
            pixel_ratio,
        }
    }
}

thread_local! {
    static LOCAL: Local = Local::new();
}
