#[cfg(target_arch = "wasm32")] // dont know how to use only one of these :(
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, prelude::*};
#[cfg(target_arch = "wasm32")]
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

#[cfg(target_arch = "wasm32")]
pub fn main() {
    log::info!(">>> worker starting");
    let scope = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()));
    let scope_clone = scope.clone();

    let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
        web_sys::console::log_1(&"got message".into());

        let mut count: u32 = 0;
        loop {
            // This would stop the main thread, but NOT with WebWorkers :)
            count += 1;
            if count >= 1_000_000_000 {
                break;
            }
        }

        let data = Array::from(&msg.data());
        let a = data
            .get(0)
            .as_f64()
            .expect("first array value to be a number") as u32;
        let b = data
            .get(1)
            .as_f64()
            .expect("second array value to be a number") as u32;

        data.push(&(a * b).into());
        scope_clone
            .post_message(&data.into())
            .expect("posting result message succeeds");
    }) as Box<dyn Fn(MessageEvent)>);
    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // The worker must send a message to indicate that it's ready to receive messages.
    scope
        .post_message(&Array::new().into())
        .expect("posting ready message succeeds");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    todo!()
}
