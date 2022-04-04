use futures::stream::{TryStreamExt, iter};
use indicatif::ProgressBar;
use std::io;
use tokio;


#[tokio::main]
async fn main() -> io::Result<()> {
    sync_loop();
    async_loop().await
}


async fn async_loop() -> io::Result<()> {
    let xs = vec![Ok(1), Ok(2)];
    let progress = ProgressBar::new(xs.len() as u64);
    let xs_iter = iter(xs);
    // TODO: This code does not compile.
    // error[E0507]: cannot move out of `progress`, a captured variable in an `FnMut` closure
    xs_iter.try_for_each(|x| async move {
        println!("{}", x);
        progress.inc(1);
        Ok(())
    }).await
}


fn sync_loop() {
    let xs = vec![1, 2];
    let progress = ProgressBar::new(xs.len() as u64);
    let xs_iter = xs.into_iter();
    xs_iter.for_each(|x| {
        println!("{}", x);
        progress.inc(1);
    });
}
