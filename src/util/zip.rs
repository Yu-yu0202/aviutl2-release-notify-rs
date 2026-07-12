use anyhow::anyhow;
use anyhow::*;
use async_zip::tokio::read::seek::ZipFileReader;
use futures_util::AsyncReadExt;
use std::path::Path;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub async fn read_txt_from_zip(zip: &Path, filepath: &str) -> Result<String> {
    let file = File::open(zip).await?;
    let reader = BufReader::new(file);

    let mut zip = ZipFileReader::new(reader.compat()).await?;
    let mut file_index = None;

    for (index, entry) in zip.file().entries().iter().enumerate() {
        if entry.filename().as_str()? == filepath {
            file_index = Some(index);
            break;
        }
    }

    if let Some(index) = file_index {
        let mut entry_reader = zip.reader_with_entry(index).await?;
        let mut contents = String::new();
        entry_reader.read_to_string(&mut contents).await?;
        anyhow::Ok(contents)
    } else {
        Err(anyhow!("File not found in zip: {}", filepath))
    }
}
