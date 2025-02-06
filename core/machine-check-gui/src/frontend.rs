mod local;
mod update;
pub mod view;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub async fn exec() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    update::update(update::Action::GetContent).await;

    //main_context.fill_rect(0., 0., 20., 20.);
    // TODO: this is just a placeholder into which the current Javascript GUI implementation will be migrated
    //a();
}

#[wasm_bindgen]
pub async fn step_verification() {
    update::update(update::Action::Step).await;
}
