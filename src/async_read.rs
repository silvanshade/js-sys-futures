use bytes::BufMut;
use futures_core::Future;
use futures_util::io::{self, AsyncBufRead, Cursor};
use js_sys::{AsyncIterator, Boolean, Reflect, Uint8Array};
use std::{
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
    pub fn new(inner: AsyncIterator) -> anyhow::Result<Self> {
        let next = JsFuture::from(inner.next().map_err(AsyncReadableError)?);
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
                    let done = Reflect::get(&object, &"done".into())?;
                    let done = done.unchecked_into::<Boolean>().value_of();
                    if done {
                        Ok(Poll::Ready(Ok(0)))
                    } else {
                        let value = Reflect::get(&object, &"value".into())?;
                        let value = value.unchecked_into::<Uint8Array>().to_vec();
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
                Poll::Pending => {
                    cx.waker().clone().wake();
                    Ok(Poll::Pending)
                },
            }
        } else {
            let amt = inner_buf.len();
            buf.put_slice(&inner_buf[.. amt]);
            Pin::new(&mut this.data).consume(amt);
            Ok(Poll::Ready(Ok(amt)))
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
