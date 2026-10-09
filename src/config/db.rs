use redis::{Client, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RedisConfig {
    uri: String,
}

impl RedisConfig {
    pub fn client(&self) -> Result<Client> {
        Client::open(self.uri()).map_err(Into::into)
    }

    pub async fn multiplexed_connection(&self) -> Result<MultiplexedConnection> {
        self.client()?
            .get_multiplexed_async_connection()
            .await
            .map_err(Into::into)
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }
}
