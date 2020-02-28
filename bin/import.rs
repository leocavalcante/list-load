use list_load::{bucket, YouCanDoIt};

async fn eval() -> YouCanDoIt {
    let base_path = std::env::var("BASE_PATH")?;
    let base_url = std::env::var("BASE_URL")?;
    let bucket = bucket()?;

    let results = bucket.list_all(base_path, None)?;

    let names = results.iter()
        .flat_map(|result| {
            let (bucket, _) = result;
            let bucket = bucket.clone();
            bucket.contents
        })
        .filter(|obj| obj.size > 0)
        .map(|obj| format!("{}/{}", base_url, obj.key));

    println!("{:?}", names.collect::<Vec<String>>());

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = eval().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}