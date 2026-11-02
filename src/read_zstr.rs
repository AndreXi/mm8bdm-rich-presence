use std::io::{self, Cursor};

pub trait ReadZStr {
    fn read_zstr(&mut self) -> io::Result<Vec<u8>>;
}

impl<T> ReadZStr for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn read_zstr(&mut self) -> io::Result<Vec<u8>> {
        let remaining = &self.get_ref().as_ref()[self.position() as usize..];

        let len = remaining
            .iter()
            .position(|&byte| byte == 0)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unterminated null-terminated string",
                )
            })?;

        let string = remaining[..len].to_vec();
        self.set_position(self.position() + len as u64 + 1); // Consume the NUL.
        Ok(string)
    }
}
