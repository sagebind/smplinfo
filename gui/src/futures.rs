use std::{future::Future, task::Poll};

use futures_lite::future::{block_on, poll_once};

pub fn poll_once_blocking<F: Future>(future: F) -> Option<F::Output> {
    block_on(poll_once(future))
}

pub struct FuturePoller<F>(Option<F>);

impl<F: Future + Unpin> FuturePoller<F> {
    pub fn poll(&mut self) -> Poll<Option<F::Output>> {
        if let Some(future) = self.0.as_mut() {
            if let Some(result) = poll_once_blocking(future) {
                self.0 = None;
                Poll::Ready(Some(result))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
}

impl<F> From<F> for FuturePoller<F> {
    fn from(future: F) -> Self {
        Self(Some(future))
    }
}
