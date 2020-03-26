use std::error::Error;
use std::fs::DirEntry;

use regex::Regex;
use s3::bucket::Bucket;

use list_load::{bucket, YouCanDoIt};

async fn upload(entry: DirEntry, bucket: Bucket, name_re: Regex) -> YouCanDoIt<String> {
    let path = entry.path();
    let path_name = path.to_str().unwrap();
    let caps = name_re.captures(path_name);

    if let None = caps {
        println!("Ignoring {}", path_name);
        return Ok(path_name.to_string());
    }

    let caps = caps.unwrap();
    let name = caps.get(1).unwrap().as_str();
    let contents = std::fs::read(&path)?;
    let base_path = std::env::var("BASE_PATH")?;
    let filename = format!("{}/{}.csv", base_path, name);

    println!("{:?}", std::fs::read(path_name)?.len());
    tokio::fs::read(path_name).await?;

    println!("Uploading {} (size {})", path_name, contents.len());
    bucket.put_object_stream(path_name, filename.as_str()).await?;

    Ok(filename)
}

async fn eval() -> YouCanDoIt {
    let mut args = std::env::args();
    let dir = std::fs::read_dir("var/")?;
    let bucket = bucket()?;
    let name_pattern = args.nth(1).unwrap();
    let name_re = Regex::new(name_pattern.as_str())?;

    let futs = dir
        .flat_map(move |result| result)
        .map(move |entry| {
            let bucket = bucket.clone();
            let name_re = name_re.clone();

            tokio::spawn(async move {
                match upload(entry, bucket, name_re).await {
                    Ok(filename) => println!("Done {}", filename),
                    Err(e) => eprintln!("Upload failed: {}", e),
                }
            })
        });

    futures::future::join_all(futs).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = eval().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
