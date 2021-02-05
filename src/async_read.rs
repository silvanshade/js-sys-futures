use bytes::BufMut;
use futures_core::Future;
use futures_util::io::{self, AsyncBufRead, Cursor};
use js_sys::{AsyncIterator, IteratorNext, Uint8Array};
use std::{
    convert::TryFrom,
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

pub struct JsAsyncRead {
    inner: AsyncIterator,
    next: JsFuture,
    data: Cursor<Vec<u8>>,
}

impl JsAsyncRead {
    /// The inner [`js_sys::AsyncIterator`] is expected to yield values of type
    /// [`js_sys::Uint8Array`].
    pub fn new(inner: AsyncIterator) -> Result<Self, JsValue> {
        let next = JsFuture::from(inner.next()?);
        let data = Default::default();
        Ok(Self { inner, next, data })
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        mut buf: &mut [u8],
    ) -> Result<Poll<std::io::Result<usize>>, JsValue> {
        let this = self.get_mut();

        let inner_buf = match Pin::new(&mut this.data).poll_fill_buf(cx) {
            Poll::Ready(Ok(buf)) => buf,
            Poll::Ready(Err(err)) => return Ok(Poll::Ready(Err(err))),
            Poll::Pending => return Ok(Poll::Pending),
        };

        if inner_buf.is_empty() {
            let next = Pin::new(&mut this.next);
            let status = next.poll(cx)?;
            match status {
                Poll::Ready(object) => {
                    let iterator_next = object.unchecked_into::<IteratorNext>();
                    if iterator_next.done() {
                        Ok(Poll::Ready(Ok(0)))
                    } else {
                        let value = iterator_next.value().dyn_into::<Uint8Array>()?.to_vec();
                        this.data = Cursor::new(value);
                        match this.inner.next() {
                            Ok(promise) => {
                                this.next = JsFuture::from(promise);
                            },
                            Err(error) => {
                                return Err(error);
                            },
                        }
                        cx.waker().clone().wake();
                        Ok(Poll::Pending)
                    }
                },
                Poll::Pending => Ok(Poll::Pending),
            }
        } else {
            let amt = std::cmp::min(inner_buf.len(), buf.len());
            buf.put_slice(&inner_buf[.. amt]);
            Pin::new(&mut this.data).consume(amt);
            Ok(Poll::Ready(Ok(amt)))
        }
    }
}

impl TryFrom<AsyncIterator> for JsAsyncRead {
    type Error = JsValue;

    fn try_from(inner: AsyncIterator) -> Result<Self, JsValue> {
        Self::new(inner)
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

impl io::AsyncRead for JsAsyncRead {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        match JsAsyncRead::poll_read(self, cx, buf) {
            Ok(success) => success,
            Err(error) => {
                let kind = io::ErrorKind::Other;
                let error = AsyncReadableError(error);
                Poll::Ready(Err(io::Error::new(kind, error)))
            },
        }
    }
}
