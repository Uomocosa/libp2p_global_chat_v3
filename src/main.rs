#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, BlobPropertyBag, Document, Url, Worker};

#[cfg(target_arch = "wasm32")]
fn worker_new(name: &str) -> Worker {
    let document: Document = web_sys::window()
        .expect("no global `window`")
        .document()
        .expect("should have a document");
    let origin = document
        .base_uri()
        .expect("base uri gives a result")
        .expect("base uri to be available");

    let script = Array::new();
    script.push(
        &format!(r#"importScripts("{origin}{name}.js");wasm_bindgen("{origin}{name}_bg.wasm");"#)
            .into(),
    );

    let options = BlobPropertyBag::new();
    options.set_type("text/javascript");
    let blob =
        Blob::new_with_str_sequence_and_options(&script, &options).expect("blob creation succeeds");

    let url = Url::create_object_url_with_blob(&blob).expect("url creation succeeds");

    Worker::new(&url).expect("failed to spawn worker")
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use wasm_bindgen::prelude::Closure;
    use web_sys::MessageEvent;
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    log::info!(">f> main");

    let web_options = eframe::WebOptions::default();

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
                Box::new(|cc| Ok(Box::new(eframe_webworker::TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
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
    });

    let worker = worker_new("worker");
    let worker_clone = worker.clone();
    log::info!("worker_clone: {worker_clone:#?}");

    // NOTE: We must wait for the worker to report that it's ready to receive
    //       messages. Any message we send beforehand will be discarded / ignored.
    //       This is different from js-based workers, which can send messages
    //       before the worker is initialized.
    //       REASON: This is because javascript only starts processing MessageEvents
    //       once the worker's script first yields to the javascript event loop.
    //       For js workers this means that you can register the event listener
    //       as first thing in the worker and will receive all previously sent
    //       message events. However, loading wasm is an asynchronous operation
    //       which yields to the js event loop before the wasm is loaded and had
    //       a change to register the event listener. At that point js processes
    //       the message events, sees that there isn't any listener registered,
    //       and drops them.

    let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
        let worker_clone = worker_clone.clone();
        let data = Array::from(&msg.data());

        if data.length() == 0 {
            let msg = Array::new();
            msg.push(&2.into());
            msg.push(&5.into());
            worker_clone
                .post_message(&msg.into())
                .expect("sending message to succeed");
        } else {
            let a = data
                .get(0)
                .as_f64()
                .expect("first array value to be a number") as u32;
            let b = data
                .get(1)
                .as_f64()
                .expect("second array value to be a number") as u32;
            let result = data
                .get(2)
                .as_f64()
                .expect("third array value to be a number") as u32;

            web_sys::console::log_1(&format!("{a} x {b} = {result}").into());
        }
    }) as Box<dyn Fn(MessageEvent)>);
    worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    log::info!(">>> main ended");
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe_webworker",
        native_options,
        Box::new(|cc| Ok(Box::new(eframe_webworker::TemplateApp::new(cc)))),
    )
}
