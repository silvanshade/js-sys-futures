use futures_util::io::AsyncReadExt;
use js_sys::*;
use js_sys_futures::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn read_uint8array() {
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

        let amt = reader.read(&mut out).await.unwrap();
        let val = [1];
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val);

        let amt = reader.read(&mut out).await.unwrap();
        let val = [2, 3];
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val);

        let amt = reader.read(&mut out).await.unwrap();
        let val = [4, 5];
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val);

        let amt = reader.read(&mut out).await.unwrap();
        let val = [6];
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val);

        let amt = reader.read(&mut out).await.unwrap();
        let val = [];
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val);

        Ok(())
    }

    run().await.unwrap();
}

#[wasm_bindgen_test]
async fn read_string() {
    async fn run() -> Result<(), Error> {
        let vals: Vec<&str> = vec!["foo", "bar", "baz"];
        let vals = vals
            .into_iter()
            .map(|str| JsString::from(str).into())
            .collect::<Vec<JsValue>>();
        let vals = vals.into_iter().collect::<Array>();
        let iter = super::create_async_iterable(&vals.values());

        let mut reader = JsAsyncRead::new(iter)?;
        let mut out = [0u8; 3];

        let amt = reader.read(&mut out).await.unwrap();
        let val = "foo";
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val.as_bytes());

        let amt = reader.read(&mut out).await.unwrap();
        let val = "bar";
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val.as_bytes());

        let amt = reader.read(&mut out).await.unwrap();
        let val = "baz";
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val.as_bytes());

        let amt = reader.read(&mut out).await.unwrap();
        let val = "";
        assert_eq!(amt, val.len());
        assert_eq!(&out[.. amt], val.as_bytes());

        Ok(())
    }

    run().await.unwrap();
}
