use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use list_load::{bucket, YouCanDoIt};

#[derive(Serialize, Deserialize)]
struct CreateListRequestList {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct CreateListRequest {
    list: CreateListRequestList,
}

#[derive(Serialize, Deserialize)]
struct IdResponse {
    id: u64,
}

#[derive(Serialize, Deserialize)]
struct ImportRequest {
    contact_import: ContactImport,
}

#[derive(Serialize, Deserialize)]
struct ContactImport {
    list_ids: Vec<u64>,
    url: String,
}

#[derive(Clone)]
struct ImportContext {
    api_url: String,
    account_id: String,
    list_name_prefix: String,
    state: String,
    file_url: String,
    client: reqwest::Client,
}

async fn import(context: ImportContext) -> YouCanDoIt {
    let create_response: IdResponse = context.client
        .post(format!("{}/accounts/{}/lists", context.api_url, context.account_id).as_str())
        .json(&CreateListRequest {
            list: CreateListRequestList {
                name: format!("{} {}", context.list_name_prefix, context.state),
                description: format!("Lista do estado {}", context.state),
            }
        })
        .send()
        .await?
        .json()
        .await?;

    let list_id = create_response.id;

    let import_response: IdResponse = context.client
        .post(format!("{}/accounts/{}/contact_imports", context.api_url, context.account_id).as_str())
        .json(&ImportRequest {
            contact_import: ContactImport {
                list_ids: [list_id].to_vec(),
                url: context.file_url.to_string(),
            },
        })
        .send()
        .await?
        .json()
        .await?;

    let import_id = import_response.id;

    println!("Import {} for list {}", import_id, list_id);

    Ok(())
}

async fn eval() -> YouCanDoIt {
    let base_path = std::env::var("BASE_PATH")?;
    let base_url = std::env::var("BASE_URL")?;
    let api_url = std::env::var("API_URL")?;
    let api_token = std::env::var("API_TOKEN")?;
    let account_id = std::env::var("ACCOUNT_ID")?;
    let list_name_prefix = std::env::var("LIST_NAME_PREFIX")?;

    let bucket = bucket()?;
    let results = bucket.list_all(base_path, None)?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-Auth-Token", reqwest::header::HeaderValue::from_str(api_token.as_str())?);
    headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let results = results.iter()
        .flat_map(|result| {
            let (bucket, _) = result;
            let bucket = bucket.clone();
            bucket.contents
        })
        .filter(|obj| obj.size > 0)
        .map(|obj| {
            let state = obj.key[11..13].to_string();
            let file_url = format!("{}/{}", base_url, &obj.key);
            (state, file_url)
        })
        .map(|(state, file_url)| {
            let context = ImportContext {
                api_url: api_url.clone(),
                account_id: account_id.clone(),
                list_name_prefix: list_name_prefix.clone(),
                state,
                file_url,
                client: client.clone(),
            };

            import(context)
        });

    let results = futures::future::try_join_all(results).await;

    println!("{:?}", results);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = eval().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}