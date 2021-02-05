use futures_util::io::AsyncReadExt;
use js_sys::*;
use js_sys_futures::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn read_exact() {
    async fn run() -> Result<(), Error> {
        let vals: Vec<&[u8]> = vec![&[1], &[2, 3], &[4, 5, 6]];
        let vals = vals
            .into_iter()
            .map(|bytes| Uint8Array::from(bytes).into())
            .collect::<Vec<JsValue>>();
        let vals = vals.into_iter().collect::<Array>();
        let iter = super::create_async_iterable(&vals.values());

        let mut reader = JsAsyncRead::new(iter)?;
        let mut out = [0u8; 2];

        let res = reader.read(&mut out).await.unwrap();
        assert_eq!(res, 1);
        assert_eq!(&out[.. res], [1]);

        let res = reader.read(&mut out).await.unwrap();
        assert_eq!(res, 2);
        assert_eq!(&out[.. res], [2, 3]);

        let res = reader.read(&mut out).await.unwrap();
        assert_eq!(res, 2);
        assert_eq!(&out[.. res], [4, 5]);

        let res = reader.read(&mut out).await.unwrap();
        assert_eq!(res, 1);
        assert_eq!(&out[.. res], [6]);

        let res = reader.read(&mut out).await.unwrap();
        assert_eq!(res, 0);
        assert_eq!(&out[.. res], []);

        Ok(())
    }

    run().await.unwrap();
}
