use deadpool::managed::Pool;
use tiberius::Client;
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use crate::{error::Error, manager::RedmondManager};

#[derive(Clone)]
pub struct SqlEngine {
    pool: Pool<Client<Compat<TcpStream>>, Error>,
}

impl SqlEngine {
    pub fn new(uri: &str) -> crate::Result<Self> {
        let manager = RedmondManager::new(uri)?;
        let pool = Pool::new(manager, 5);

        Ok(Self { pool })
    }

    pub async fn select_1(&self) -> crate::Result<i32> {
        let mut conn = self.pool.get().await?;

        let row = conn.query("SELECT 1", &[]).await?.into_row().await?.unwrap();

        Ok(row.get(0).unwrap())
    }
}
