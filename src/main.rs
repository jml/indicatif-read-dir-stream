use core::pin::Pin;
use futures::stream::{self, TryStreamExt, iter};
use futures::task::{Context, Poll};
use indicatif::ProgressBar;
use std::io;
use tokio;


#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Synchronous bar");
    sync_loop();
    println!("Async bar");
    async_loop().await
}


struct ProgressStream<S: stream::Stream> {
    progress_bar: ProgressBar,
    stream: S,
}


impl<S: stream::Stream + std::marker::Unpin> stream::Stream for ProgressStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                self.progress_bar.finish();
                Poll::Ready(None)
            }
            Poll::Ready(Some(x)) => {
                self.progress_bar.inc(1);
                Poll::Ready(Some(x))
            }
        }
    }
}


async fn async_loop() -> io::Result<()> {
    let xs = vec![Ok(1), Ok(2)];
    let progress = ProgressBar::new(xs.len() as u64);
    let progress_stream = ProgressStream { progress_bar: progress, stream: iter(xs) };
    // TODO: This code does not compile.
    // error[E0507]: cannot move out of `progress`, a captured variable in an `FnMut` closure
    progress_stream.try_for_each(|_x| async move {
        Ok(())
    }).await
}


fn sync_loop() {
    let xs = vec![1, 2];
    let progress = ProgressBar::new(xs.len() as u64);
    let xs_iter = xs.into_iter();
    xs_iter.for_each(|_x| {
        progress.inc(1);
    });
    progress.finish();
}
