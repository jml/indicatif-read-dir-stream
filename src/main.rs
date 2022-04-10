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
    async_loop().await?;
    println!("Working async bar");
    working_async_loop().await?;
    println!("Pinned & working async bar");
    pinned_async_loop().await
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


async fn working_async_loop() -> io::Result<()> {
    let xs: Vec<Duration> = vec![2000, 1000, 3000, 4000].into_iter().map(Duration::from_millis).collect();
    let runner = ProgressRunner::new();
    runner.run(4, xs).await
}


struct ProgressRunner {
    progress_bar: ProgressBar,
}

impl ProgressRunner {
    fn new() -> Self {
        ProgressRunner {progress_bar: ProgressBar::new(0)}
    }

    async fn run(&self, limit: usize, times: Vec<Duration>) -> io::Result<()> {
        // If we make the progress bar a member of the struct, we do not encounter this moving problem.
        // TODO(jml): I wonder if pinning is the answer.
        self.progress_bar.set_length(times.len() as u64);
        let stream = iter(times.into_iter().map(|x| -> io::Result<Duration> { Ok(x) }));
        stream.try_for_each_concurrent(limit, |x| async move {
            time::sleep(x).await;
            self.progress_bar.inc(1);
            Ok(())
        }).await
    }
}


/// Rather than requiring the progress bar to be a field of the struct,
/// we *pin* it, so that it does not move.
async fn pinned_async_loop() -> io::Result<()> {
    let times: Vec<Duration> = vec![2000, 1000, 3000, 4000].into_iter().map(Duration::from_millis).collect();
    let progress_bar = ProgressBar::new(times.len() as u64);
    let progress_bar = Pin::new(&progress_bar);
    let stream = iter(times.into_iter().map(|x| -> io::Result<Duration> { Ok(x) }));
    stream.try_for_each_concurrent(4, |x| async move {
        time::sleep(x).await;
            progress_bar.inc(1);
        Ok(())
    }).await
}
