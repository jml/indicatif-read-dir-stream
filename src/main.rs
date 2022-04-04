use futures::stream::{TryStreamExt, iter};
use indicatif::ProgressBar;
use std::io;
use tokio;


#[tokio::main]
async fn main() -> io::Result<()> {
    let xs = vec![Ok(1), Ok(2)];
    let xs_iter = iter(xs);
    // TODO: It is hilarious how much code it is to pass the length of the vector to the progress bar.
    // I am too tired to figure it out right now.
    let progress = ProgressBar::new(2);
    // TODO: This code does not compile.
    // error[E0507]: cannot move out of `progress`, a captured variable in an `FnMut` closure
    xs_iter.try_for_each_concurrent(1, |x| async move {
        println!("{}", x);
        progress.inc(1);
        Ok(())
    }).await
}
