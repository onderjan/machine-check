use log::info;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub struct Gui {
    event_loop: tao::event_loop::EventLoop<()>,
    #[allow(dead_code)]
    window: tao::window::Window,
    #[allow(dead_code)]
    web_view: wry::WebView,
}

type ResponseFn = http::Response<std::borrow::Cow<'static, [u8]>>;

impl Gui {
    pub fn new(
        response_fn: fn(http::Request<Vec<u8>>) -> ResponseFn,
    ) -> Result<Gui, Box<dyn std::error::Error>> {
        // build the GUI using the packages wry and tao
        let event_loop = EventLoop::new();
        // TODO: set a reasonable title that changes depending on the binary
        let window = WindowBuilder::new()
            .with_title("machine-check GUI")
            .build(&event_loop)?;

        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))]
        let builder = WebViewBuilder::new(&window);

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        )))]
        let builder = {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            let vbox = window.default_vbox()?;
            WebViewBuilder::new_gtk(vbox)
        };
        let web_view = builder
            .with_custom_protocol("gui".into(), response_fn)
            // tell the webview to load the custom protocol
            .with_url("gui://localhost")
            .build()?;
        Ok(Gui {
            event_loop,
            window,
            web_view,
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
