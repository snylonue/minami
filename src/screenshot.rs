use scrap::{Capturer as RealCapturer, Display};

pub struct Screenshot {
    pub capturer: RealCapturer,
}

impl Screenshot {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            capturer: RealCapturer::new(Display::primary()?)?,
        })
    }
    pub fn capturer_size(&self) -> (usize, usize) {
        (self.capturer.width(), self.capturer.height())
    }
    pub fn capture(&mut self) -> Result<Vec<u8>, std::io::Error> {
        loop {
            match self.capturer.frame() {
                Ok(frame) => break Ok(frame.to_vec()), // sometimes returns an empty frame
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => break Err(e),
            }
        }
    }
}
