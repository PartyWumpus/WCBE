mod app;
mod befunge;
mod befunge93;
mod befunge93mini;
mod befunge98;

use app::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::app::{Settings, StartConfig};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn render_to_canvas(
    canvas: web_sys::HtmlCanvasElement,
    program: &str,
    starting_position: &[i16],
) {
    let web_options = eframe::WebOptions::default();

    let starting_position = Some((starting_position[0] as i64, starting_position[1] as i64));

    let args = StartConfig {
        contents: Some(program.to_string()),
        fix_camera_to_program: true,
        start_in_run_mode: true,
        hide_toolbar: true,
        hide_extra: true,
        settings: Some(Settings::default()),
        starting_position,
        ..Default::default()
    };

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(crate::App::new(cc, args)))),
            )
            .await;

        // Remove the loading text and spinner:
        /*
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
        */
    });
}
