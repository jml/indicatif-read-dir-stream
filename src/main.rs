use futures::stream::{self, TryStreamExt, iter};
use futures::task::{Context, Poll};
use indicatif::ProgressBar;
use pin_project_lite::pin_project;
use std::io;
use std::pin::Pin;
use tokio;
use tokio::time::{self, Duration};


#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Synchronous bar");
    sync_loop();
    println!("Async bar");
    async_loop().await
}


pin_project! {
    struct ProgressStream<S: stream::Stream> {
        progress_bar: ProgressBar,
        #[pin]
        stream: S,
    }
}


impl<S: stream::Stream> stream::Stream for ProgressStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let result = this.stream.as_mut().poll_next(ctx);
        match &result {
            Poll::Pending => (),
            Poll::Ready(None) => {
                this.progress_bar.finish();
            }
            Poll::Ready(_) => {
                this.progress_bar.inc(1);
            }
        };
        result
    }
}


/// A bad example of how to update a progress bar by consuming a stream.
///
/// Each of the items in the stream will be processed serially, with no concurrency.
/// This is because streams only let you get the first item from the stream when it's ready,
/// which is quite reasonable when you think about it.
async fn async_loop() -> io::Result<()> {
    use futures::FutureExt;

    let xs = vec![Ok(2000), Ok(1000), Ok(3000), Ok(4000)];
    let progress = ProgressBar::new(xs.len() as u64);
    let stream = iter(xs).and_then(|x| { time::sleep(Duration::from_millis(x)).map(Ok) });
    let progress_stream = ProgressStream { progress_bar: progress, stream: stream };
    progress_stream.try_for_each_concurrent(4, |_x| async move {
        Ok(())
    }).await
}


/// An example of updating a progress bar with purely synchronous logic.
fn sync_loop() {
    let xs = vec![1, 2];
    let progress = ProgressBar::new(xs.len() as u64);
    let xs_iter = xs.into_iter();
    xs_iter.for_each(|_x| {
        progress.inc(1);
    });
    progress.finish();
}


// A bad example of how to update a progress bar by consuming a stream.
//
// Each of the items in the stream will be processed serially, with no concurrency.
// This is because streams only let you get the first item from the stream when it's ready,
// which is quite reasonable when you think about it.
//
// async fn broken_async_loop() {
//     let xs = vec![Ok(2000), Ok(1000), Ok(3000), Ok(4000)];
//     let progress = ProgressBar::new(xs.len() as u64);
//     let stream = iter(xs);
//     stream.try_for_each_concurrent(4, |x| async move {
//         time::sleep(Duration::from_millis(x)).await;
//         progress.inc(1);
//         Ok(())
//     }).await
// }
