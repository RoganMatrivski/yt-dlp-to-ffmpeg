use std::io::Write;
use std::task::Poll;
use std::{io, pin::pin};

use tokio::io::AsyncWrite;

pub struct Md5Writer<T> {
    writer: T,
    context: md5::Context,
}

impl<T: Write> Md5Writer<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer,
            context: md5::Context::new(),
        }
    }

    pub fn md5(self) -> String {
        format!("{:x}", self.context.compute())
    }
}

impl<T: AsyncWrite> Md5Writer<T> {
    pub fn new_async(writer: T) -> Self {
        Self {
            writer,
            context: md5::Context::new(),
        }
    }
}

impl<T: AsyncWrite + std::marker::Unpin> AsyncWrite for Md5Writer<T> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        let Self {
            writer, context, ..
        } = self.get_mut();
        match pin!(writer).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => {
                context.consume(&buf[..n]);
                Poll::Ready(Ok(n))
            }
            poll => poll,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        let Self { writer, .. } = self.get_mut();
        pin!(writer).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        let Self { writer, .. } = self.get_mut();
        pin!(writer).poll_shutdown(cx)
    }
}

impl<T: Write> Write for Md5Writer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let byte_count = self.writer.write(buf)?;
        self.context.consume(&buf[..byte_count]);
        Ok(byte_count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
