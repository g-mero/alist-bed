use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use anyhow::anyhow;
use reqwest::header;
use serde::{Deserialize, Serialize};

pub use error::Error;

use crate::error::AppResult;

mod error;

pub struct AlistApi {
    host: String,
    client: reqwest::Client,
}

pub fn new(token: &str, host: &str) -> AlistApi {
    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", (&token).parse().unwrap());
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build().unwrap();
    AlistApi { host: host.to_string(), client }
}

fn read_file_to_vec<P: AsRef<Path>>(path: P) -> io::Result<(Vec<u8>, String)> {
    // 打开文件
    let mut file = File::open(&path)?;

    // 获取文件名
    let file_name = match path.as_ref().file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(io::Error::new(io::ErrorKind::Other, "No filename found")),
    };

    // 读取文件内容到Vec<u8>
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok((buffer, file_name))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImgInfo {
    pub name: String,
    pub raw_url: String,
}

impl AlistApi {
    pub async fn start(&self) {
        let resp = self.client.get(format!("{}/api/me", &self.host)).send().await.unwrap();
        println!("{:?}", resp.text().await.unwrap());
    }

    pub async fn img_raw_data(&self, remote_path: &str) -> AppResult<Vec<u8>> {
        let url = self.img_info(remote_path).await?.raw_url;
        let data = self.client.get(&url).send().await?.bytes().await?;
        Ok(data.to_vec())
    }

    pub async fn img_info(&self, remote_path: &str) -> AppResult<ImgInfo> {
        let url = format!("{}/api/fs/get?path={}", &self.host, remote_path);
        let resp = self.client.post(url).send().await?;
        let json = resp.json::<serde_json::Value>().await?;
        if json["code"].as_i64().unwrap_or_default() != 200 {
            return Err(anyhow!("request failed").into());
        }
        let name = json["data"]["name"].as_str()
            .ok_or(anyhow!("cant get filename"))?.to_string();
        let raw_url = json["data"]["raw_url"].as_str()
            .ok_or(anyhow!("cant get raw url"))?.to_string();

        Ok(ImgInfo { name, raw_url })
    }

    pub async fn img_upload(&self, remote_path: &str, local_path: &str) {
        let (file_content, file_name) = read_file_to_vec(local_path).unwrap();


        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name));
        let url = format!("{}/api/fs/form", &self.host);
        let resp = self.client.put(url)
            .multipart(form)
            .header("File-Path", remote_path)
            .send().await.unwrap();
        println!("{:?}", resp.text().await.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start() {
        let api = new("alist-d7e33330-03ec-4598-8cb3-4a461731909fBcrWpMQcqsMTs3TTq3OPMIjyBU2tDWyFbgt1WwNz2VQ1OBx2V1vol3e3JeOY4ngA", "https://pan.gmero.com");
        api.start().await;
    }

    #[tokio::test]
    async fn test_img_info() {
        let api = new("alist-d7e33330-03ec-4598-8cb3-4a461731909fBcrWpMQcqsMTs3TTq3OPMIjyBU2tDWyFbgt1WwNz2VQ1OBx2V1vol3e3JeOY4ngA",
                      "https://pan.gmero.com");
        api.img_info("/onedrive/assets/fonts/BF%E7%B3%BB%E5%88%97%E5%AD%97%E4%BD%93.zip").await.expect("TODO: panic message");
    }

    #[tokio::test]
    async fn test_img_upload() {
        let api = new("alist-1764b6ff-4647-49ef-99f1-e585d80dec96M7qakg87hpHEdGjX9HQN4LnRrB1808P2kIZyCY9e7Kx3sR4r0rGMJAgbo4n3p0GX",
                      "https://pan.gmero.com");
        api.img_upload("/onedrive/test/test.jpg", "../test.jpg").await;
    }
}