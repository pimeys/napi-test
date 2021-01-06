use async_trait::async_trait;
use deadpool::managed::{Manager, RecycleResult};
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::Error;

pub struct RedmondManager {
    config: Config,
}

impl RedmondManager {
    pub fn new(url: &str) -> crate::Result<Self> {
        let config = Config::from_ado_string(url)?;

        Ok(Self { config })
    }
}

#[async_trait]
impl Manager<Client<Compat<TcpStream>>, Error> for RedmondManager {
    async fn create(&self) -> crate::Result<Client<Compat<TcpStream>>> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(self.config.clone(), tcp.compat_write()).await?;

        Ok(client)
    }

    async fn recycle(&self, _: &mut Client<Compat<TcpStream>>) -> RecycleResult<Error> {
        Ok(())
    }
}
