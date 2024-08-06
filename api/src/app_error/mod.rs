use std::io;
use std::string::FromUtf8Error;

use async_trait::async_trait;
use salvo::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io: `{0}`")]
    Io(#[from] io::Error),
    #[error("utf8: `{0}`")]
    FromUtf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error("alist: `{0}`")]
    Alist(#[from] alist_api::Error),
    #[error("salvo:`{0}`")]
    Salvo(#[from] salvo::Error),
}

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render(Text::Plain(self.to_string()));
    }
}

pub type AppResult<T> = Result<T, AppError>;