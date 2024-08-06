use std::sync::LazyLock;

use anyhow::anyhow;
use salvo::http::HeaderMap;
use salvo::prelude::*;
use time::OffsetDateTime;

use alist_api::AlistApi;

use crate::app_error::AppResult;

mod app_error;

static ALIST_SDK: LazyLock<AlistApi> = LazyLock::new(|| {
    let conf = config::get_config();
    alist_api::new(&conf.alist_token, &conf.alist_host)
});

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[handler]
async fn get_img(req: &mut Request, res: &mut Response) -> AppResult<()> {
    let mut img_path = req.param::<String>("*+img_path")
        .ok_or(anyhow!("img_path not found"))?;

    let conf = config::get_config();
    img_path = format!("{}/{img_path}", &conf.alist_dir);

    let is_thumb = req.query::<String>("size").unwrap_or_default() == "small";
    if is_thumb {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Disposition", "inline".parse().unwrap());
        let support_webp = req.header::<String>("Accept").unwrap_or_default().contains("image/webp");
        if support_webp {
            headers.insert("Content-Type", "image/webp".parse().unwrap());
        } else {
            headers.insert("Content-Type", "image/jpeg".parse().unwrap());
        }
        let cache_key = format!("{}-thumb-{}", img_path, support_webp);
        let img = cache::get(&cache_key);
        return match img {
            None => {
                let mut img = ALIST_SDK.img_raw_data(&img_path).await?;
                if support_webp {
                    img = img_process::to_webp(&img)?;
                } else {
                    img = img_process::only_thumbnail(&img)?;
                }

                cache::set(cache_key, img.clone());
                res.write_body(img)?;
                Ok(())
            }
            Some(i) => {
                res.write_body(i)?;
                Ok(())
            }
        };
    }

    let img_info = ALIST_SDK.img_info(&img_path).await?;
    res.render(Redirect::temporary(img_info.raw_url));

    Ok(())
}

#[handler]
async fn upload_img(req: &mut Request, res: &mut Response) -> AppResult<()> {
    let file = req.file("file").await.ok_or(anyhow!("file not found"))?;

    let conf = config::get_config();

    // make remote path like {alist_dir}/{year}/{timestamp}.{ext}
    let file_name = file.name().unwrap();
    let ext = file_name.split('.').last().unwrap();
    let now = OffsetDateTime::now_utc();
    let remote_path = format!("{}/{}/{}.{}", conf.alist_dir, now.year(), now.unix_timestamp(), ext);

    ALIST_SDK.img_upload(&remote_path, file.path().to_str().unwrap()).await;

    res.render("upload success");
    Ok(())
}

#[handler]
async fn check_key(req: &mut Request, res: &mut Response) {
    let key = req.header::<String>("API-KEY");

    match key {
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render("API-KEY not found");
        }
        Some(k) => {
            let conf = config::get_config();
            if k != conf.api_key {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render("API-KEY not correct");
            }
        }
    }
}

#[handler]
async fn clear_cache(_req: &mut Request, res: &mut Response) {
    cache::clear();

    res.render("ok");
}

pub async fn start() {
    tracing_subscriber::fmt().init();

    let router = Router::new().get(hello)
        .push(Router::with_path("pic/<*+img_path>").get(get_img))
        .push(
            Router::new().hoop(check_key)
                .push(Router::with_path("upload").post(upload_img))
                .push(Router::with_path("clear").post(clear_cache))
        );
    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}