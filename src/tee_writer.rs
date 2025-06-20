use std::io::{self, Write};

/// General purpose tee writer.
///
///Writes given data to both writers.
pub struct TeeWriter<W1: Write, W2: Write> {
    writer1: W1,
    writer2: W2,
}

impl<W1: Write, W2: Write> TeeWriter<W1, W2> {
    pub fn new(writer1: W1, writer2: W2) -> Self {
        Self { writer1, writer2 }
    }

    pub fn into_inner(self) -> (W1, W2) {
        (self.writer1, self.writer2)
    }
}

impl<W1: Write, W2: Write> Write for TeeWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n1 = self.writer1.write(buf)?;
        let n2 = self.writer2.write(&buf[..n1])?;
        // Return the minimum bytes written
        Ok(n1.min(n2))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer1.flush()?;
        self.writer2.flush()?;
        Ok(())
    }
}
