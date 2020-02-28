use std::error::Error;

use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;

pub type YouCanDoIt<T = ()> = Result<T, Box<dyn Error>>;

pub fn bucket() -> YouCanDoIt<Bucket> {
    let bucket_name = std::env::var("S3_BUCKET")?;
    let s3_access_key = std::env::var("S3_ACCESS_KEY")?;
    let s3_secret = std::env::var("S3_SECRET")?;
    let s3_credentials = Credentials::new(Some(s3_access_key), Some(s3_secret), None, None);
    let s3_endpoint = std::env::var("S3_ENDPOINT")?;
    let region = Region::Custom { region: "us-east-1".to_string(), endpoint: s3_endpoint };

    Ok(Bucket::new(bucket_name.as_ref(), region, s3_credentials)?)
}