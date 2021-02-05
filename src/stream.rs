use futures_core::{Future, Stream};
use js_sys::{AsyncIterator, IteratorNext};
use std::{
    convert::TryFrom,
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
    pub fn new(inner: AsyncIterator) -> Result<Self, JsValue> {
        let next = JsFuture::from(inner.next()?);
        let phantom = std::marker::PhantomData;
        Ok(Self { inner, next, phantom })
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Result<Poll<Option<T>>, JsValue> {
        let this = self.get_mut();
        let next = Pin::new(&mut this.next);
        let status = next.poll(cx)?;
        match status {
            Poll::Ready(object) => {
                let iterator_next = object.unchecked_into::<IteratorNext>();
                if iterator_next.done() {
                    Ok(Poll::Ready(None))
                } else {
                    let value = iterator_next.value().unchecked_into::<T>();
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
            Poll::Pending => Ok(Poll::Pending),
        }
    }
}

impl<T: Unpin + JsCast> TryFrom<AsyncIterator> for JsStream<T> {
    type Error = JsValue;

    fn try_from(inner: AsyncIterator) -> Result<Self, JsValue> {
        Self::new(inner)
    }
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
