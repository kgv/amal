#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use amal::App;

// Native
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    std::env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "2");
    // std::env::set_var("TRIM_DECIMAL_ZEROS", "256");

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = Default::default();
    eframe::run_native(
        "MS viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// Web
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = Default::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(error) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {error:?}");
                }
            }
        }
    });
}
