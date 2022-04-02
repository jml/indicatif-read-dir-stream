use futures::stream::TryStreamExt;
use indicatif::ProgressBar;
use std::convert::TryInto;
use std::io;
use std::path::Path;
use std::fs as std_fs;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;


#[tokio::main]
async fn main() -> io::Result<()> {
    let source_dir = Path::new("/");
    let num_files = std_fs::read_dir(&source_dir)?.count();
    let progress = ProgressBar::new(num_files.try_into().unwrap());
    let source_dir = fs::read_dir(&source_dir).await?;
    let stream = ReadDirStream::new(source_dir);
    let num_processes = 4;
    stream
        .try_for_each_concurrent(num_processes, |_source_file| async move {
            progress.inc(1);
            Ok(())
        }).await?;
    Ok(())
}
