use std::fs;

use serde::{Deserialize, Serialize};

mod markdown;
mod schedule;
mod summary;
mod the_movie_db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = serde_gura::from_str(&fs::read_to_string("config.gura")?)?;

    if config.tv_ids.is_empty() {
        return Err("empty tv_ids".into());
    };

    let client = the_movie_db::Client::new(config.api_key);

    let tv = client.get_tvs(config.tv_ids).await?;

    let summary = summary::Summary::new(schedule::now(), tv, &config.streaming_networks);

    let md = markdown::to_markdown_github(&summary);
    println!("{}", md);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    api_key: String,
    streaming_networks: Vec<u32>,
    tv_ids: Vec<u32>,
}
