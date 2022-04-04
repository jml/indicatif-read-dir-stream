use futures::stream::{TryStreamExt, iter};
use indicatif::ProgressBar;
use std::io;
use tokio;


#[tokio::main]
async fn main() -> io::Result<()> {
    let xs = vec![Ok(1), Ok(2)];
    let xs_iter = iter(xs);
    let progress = ProgressBar::new(4);
    xs_iter.try_for_each_concurrent(1, |x| async move {
        println!("{}", x);
        progress.inc(1);
        Ok(())
    }).await
}
