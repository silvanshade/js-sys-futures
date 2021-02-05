use wasm_bindgen::prelude::*;

mod async_read;
mod stream;

#[wasm_bindgen(module = "tests/wasm/async_iterable.js")]
extern {
    #[wasm_bindgen(js_name = createAsyncIterable)]
    fn create_async_iterable(iterable: &js_sys::Iterator) -> js_sys::AsyncIterator;
}
