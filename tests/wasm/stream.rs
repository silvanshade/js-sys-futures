use futures_util::stream::StreamExt;
use js_sys::*;
use js_sys_futures::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn stream() {
    async fn run() -> Result<(), Error> {
        let vals = vec!["foo", "bar", "baz"]
            .into_iter()
            .map(Into::into)
            .collect::<Vec<JsValue>>();
        let vals = vals.into_iter().collect::<Array>();
        let iter = super::create_async_iterable(&vals.values());

        let mut stream = JsStream::<JsString>::new(iter)?;

        if let Some(result) = stream.next().await {
            assert_eq!(JsString::from("foo"), result?);
        }
        if let Some(result) = stream.next().await {
            assert_eq!(JsString::from("bar"), result?);
        }
        if let Some(result) = stream.next().await {
            assert_eq!(JsString::from("baz"), result?);
        }

        Ok(())
    }

    run().await.unwrap();
}
