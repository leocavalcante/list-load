use std::error::Error;
use std::fs::DirEntry;

use s3::bucket::Bucket;

use list_load::{bucket, YouCanDoIt};

fn upload(entry: DirEntry, bucket: Bucket) -> YouCanDoIt<String> {
    let path = entry.path();
    let state = path.as_os_str().to_str().unwrap();
    let state = &state[10..12];
    let state = state.to_uppercase();
    let contents = std::fs::read(path)?;
    let base_path = std::env::var("BASE_PATH")?;
    let filename = format!("{}/{}.csv", base_path, state);

    println!("Uploading {} (size {})", filename, contents.len());

    let _result = bucket.put_object(filename.as_ref(), contents.as_ref(), "text/plain")?;

    Ok(filename)
}

async fn eval() -> YouCanDoIt {
    let dir = std::fs::read_dir("var/")?;
    let bucket = bucket()?;

    dir.flat_map(|result| result)
        .for_each(|entry| {
            let bucket = bucket.clone();
            tokio::task::spawn_blocking(move || {
                match upload(entry, bucket) {
                    Ok(filename) => println!("Done {}", filename),
                    Err(e) => eprintln!("Upload failed: {}", e),
                }
            });
        });

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = eval().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
