use futures_core::{Future, Stream};
use js_sys::{AsyncIterator, Boolean, Reflect};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

pub struct JsStream<T: Unpin + JsCast> {
    inner: AsyncIterator,
    next: JsFuture,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Unpin + JsCast> JsStream<T> {
    pub fn new(inner: AsyncIterator) -> anyhow::Result<Self> {
        let next = JsFuture::from(inner.next().map_err(AsyncReadableError)?);
        let phantom = std::marker::PhantomData;
        Ok(Self { inner, next, phantom })
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Result<Poll<Option<T>>, JsValue> {
        let this = self.get_mut();
        let next = Pin::new(&mut this.next);
        let status = next.poll(cx)?;
        match status {
            Poll::Ready(object) => {
                let done = Reflect::get(&object, &"done".into())?;
                let done = done.unchecked_into::<Boolean>().value_of();
                if done {
                    Ok(Poll::Ready(None))
                } else {
                    let value = Reflect::get(&object, &"value".into())?;
                    let value = value.unchecked_into::<T>();
                    match this.inner.next() {
                        Ok(promise) => {
                            this.next = JsFuture::from(promise);
                        },
                        Err(error) => {
                            return Err(error);
                        },
                    }
                    Ok(Poll::Ready(Some(value)))
                }
            },
            Poll::Pending => {
                cx.waker().clone().wake();
                Ok(Poll::Pending)
            },
        }
    }
}

#[derive(Clone, Debug)]
struct AsyncReadableError(JsValue);

unsafe impl Send for AsyncReadableError {
}
unsafe impl Sync for AsyncReadableError {
}

impl std::fmt::Display for AsyncReadableError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self.0)
    }
}

impl std::error::Error for AsyncReadableError {
}

impl<T: Unpin + JsCast> Stream for JsStream<T> {
    type Item = Result<T, JsValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match JsStream::poll_next(self, cx) {
            Ok(success) => success.map(|x| x.map(Ok)),
            Err(error) => Poll::Ready(Some(Err(error))),
        }
    }
}
