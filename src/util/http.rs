use anyhow::*;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

pub async fn get_url_content(url: &str, client: &Client) -> Result<String> {
    let res = client.get(url).send().await?.text().await?;

    Ok(res)
}

pub async fn download_file(url: &str, path: &Path, client: &Client) -> Result<()> {
    let res = client.get(url).send().await?;
    let file = File::create(path).await?;
    let mut writer = BufWriter::new(file);

    let mut stream = res.bytes_stream();
    while let Some(item) = StreamExt::next(&mut stream).await {
        let chunk = item?;

        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;
    Ok(())
}
