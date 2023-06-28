pub mod assets;
pub mod contracts;
pub mod pools;

use reqwest::Client;
use serde_json::Value;
use tokio::runtime::Runtime;

const ANS_SCRAPE_URL: &str =
    "https://raw.githubusercontent.com/AbstractSDK/ans-scraper/mainline/out/";

/// get some json  
pub fn get_scraped_json_data(suffix: &str) -> Value {
    let client = Client::new();
    let url = format!("{}{}.json", ANS_SCRAPE_URL, suffix);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let response = client.get(&url).send().await.unwrap();
        let json: Value = response.json().await.unwrap();
        return json;
    })
}
