/// A lightweight reading buffer.
pub struct ReadBuf<const N: usize> {
    idx: usize,
    buffer: [u8; N],
}

impl<const N: usize> ReadBuf<{ N }> {
    /// Returns a new buffer.
    pub fn new() -> Self {
        let idx = 0;
        let buffer = [0u8; N];

        Self { idx, buffer }
    }

    /// Push the given byte to the buffer.
    pub fn push(&mut self, value: u8) -> Result<(), ()> {
        if self.idx < N {
            self.buffer[self.idx] = value;
            self.idx += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Returns a mutable slice of the buffer up to the last pushed byte.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.idx]
    }

    /// Resets the buffer.
    ///
    /// It does not overwrite it with zeros, therefore it is really cheap.
    pub fn reset(&mut self) {
        self.idx = 0;
    }
}
