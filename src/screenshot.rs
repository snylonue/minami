use std::{future::Future, task::Poll};

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
}

impl Future for Screenshot {
    type Output = Result<Vec<u8>, std::io::Error>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.capturer.frame() {
            Ok(frame) => Poll::Ready(Ok(frame.to_vec())), // sometimes returns an empty frame
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}


/// SAFETY: remain to be proven
unsafe impl Send for Screenshot {}