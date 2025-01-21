use indicatif::MultiProgress;
use std::io::Write;

pub struct ProgressBarLogWriter<'a, W: Write> {
    writer: W,
    mpb: &'a MultiProgress,
}

impl<'a, W: Write> ProgressBarLogWriter<'a, W> {
    pub fn new(writer: W, mpb: &'a MultiProgress) -> Self {
        Self { writer, mpb }
    }
}

impl<W: Write> Write for ProgressBarLogWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.mpb.suspend(|| self.writer.write(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.mpb.suspend(|| self.writer.flush())
    }
}
