use log::info;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};
use wry::{WebViewBuilder, WebViewId};

use super::FAVICON_ICO;

pub struct Window {
    event_loop: tao::event_loop::EventLoop<()>,
    #[allow(dead_code)]
    window: tao::window::Window,
    #[allow(dead_code)]
    web_view: wry::WebView,
}

type ResponseCow = http::Response<std::borrow::Cow<'static, [u8]>>;

impl Window {
    pub fn new(
        response_fn: impl Fn(WebViewId, http::Request<Vec<u8>>) -> ResponseCow + 'static,
        exec_name: &str,
    ) -> Result<Window, Box<dyn std::error::Error>> {
        // build the GUI using the packages wry and tao
        let event_loop = EventLoop::new();

        // to avoid external packages, just get the raw favicon data
        //knowing it is ICO containing 32px raw BGRA
        // the 6-byte ICO header will be followed by one image, get the offset
        let offset = u32::from_le_bytes(
            FAVICON_ICO[6 + 12..6 + 16]
                .try_into()
                .expect("Favicon should have a four-byte offset field"),
        );

        let icon_height = 32;
        let icon_width = 32;
        let icon_bgra_length = icon_height * icon_width * 4;
        // the raw BGRA data starts after a 40-byte header
        let icon_bgra_start = offset as usize + 40;
        let icon_bgra = &FAVICON_ICO[icon_bgra_start..icon_bgra_start + icon_bgra_length];

        let icon_rgba: Vec<u8> = icon_bgra
            .chunks_exact(4)
            .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
            .collect();

        let window = WindowBuilder::new()
            .with_title(format!("{} (machine-check)", exec_name))
            .with_maximized(true)
            .with_window_icon(Some(
                Icon::from_rgba(icon_rgba, icon_width as u32, icon_height as u32)
                    .expect("Favicon should be loaded"),
            ))
            .build(&event_loop)?;

        let builder = WebViewBuilder::new();

        let builder = builder
            .with_custom_protocol("gui".into(), response_fn)
            // tell the webview to load the custom protocol
            .with_url("gui://localhost");
        #[cfg(not(target_os = "linux"))]
        let webview = builder.build(&window)?;
        #[cfg(target_os = "linux")]
        let webview = builder.build_gtk(window.gtk_window())?;
        Ok(Window {
            event_loop,
            window,
            web_view: webview,
        })
    }

    pub fn run(self) -> ! {
        // run the event loop, hijacking the thread
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => info!("GUI window opened"),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}
