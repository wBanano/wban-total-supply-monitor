use anyhow::{Context, Result};
use redis::{Client, Commands, IntoConnectionInfo};
use rust_decimal::Decimal;
use thiserror::Error;
use tokio_postgres::NoTls;

pub enum Bridge {
    Redis(Client),
    Postgres(tokio_postgres::Client),
}

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Decimal error")]
    Decimal(#[from] rust_decimal::Error),
}

pub struct PostgresSettings {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: Option<String>,
}

impl Bridge {

    pub fn redis(redis_host: String, redis_password: Option<String>) -> Bridge {
        let mut connection_info = redis_host.into_connection_info().unwrap();
        if let Some(password) = redis_password {
            connection_info.redis.password = Some(password);
        }
        Bridge::Redis(Client::open(connection_info).unwrap())
    }

    pub async fn postgres(settings: PostgresSettings) -> Result<Bridge> {
        let mut config = tokio_postgres::Config::new();
        config
            .host(&settings.host)
            .port(settings.port)
            .dbname(&settings.dbname)
            .user(&settings.user)
            .options("-c default_transaction_read_only=on");
        if let Some(password) = &settings.password {
            config.password(password);
        }
        let (client, connection) = config.connect(NoTls).await.context("Can't connect to the ledger database")?;
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                eprintln!("Ledger database connection failed: {}", err);
            }
        });
        Ok(Bridge::Postgres(client))
    }

    pub async fn get_unwrapped_ban_balance(&self) -> Result<Decimal> {
        match self {
            Bridge::Redis(client) => Bridge::redis_unwrapped_ban_balance(client),
            Bridge::Postgres(client) => {
                let row = client
                    .query_one("select coalesce(sum(balance), 0)::text from accounts", &[])
                    .await?;
                Ok(convert_raw_balance(row.get(0))?)
            }
        }
    }

    pub async fn count_stuck_withdrawals(&self) -> Result<i64> {
        match self {
            Bridge::Redis(_) => Ok(0),
            Bridge::Postgres(client) => {
                let row = client
                    .query_one(
                        "select count(*) from ledger_entries where kind = 'withdrawal' and status = 'reserved' and created_at < now() - interval '10 minutes'",
                        &[],
                    )
                    .await?;
                Ok(row.get(0))
            }
        }
    }

    fn redis_unwrapped_ban_balance(client: &Client) -> Result<Decimal> {
        let mut con = client.get_connection()?;
        let balances_keys: Vec<String> = con.keys("ban-balance:*")?;
        let unwrapped: Decimal = balances_keys
            .into_iter()
            // map Redis key to BAN balance with 18 digits as integer
            .map(|key| con.get(key).unwrap())
            // map BAN balance to Decimal with 18 digits
            .map(|raw| convert_raw_balance(raw).unwrap())
            .sum();

        Ok(unwrapped)
    }

}

fn convert_raw_balance(raw_balance: String) -> Result<Decimal, BridgeError> {
    if raw_balance == "0" /*|| raw_balance.len() <= 11*/ {
        return Ok(Decimal::from(0))
    }
    let mut balance: Decimal = Decimal::from_str_radix(raw_balance.as_str(), 10).unwrap();
    balance.set_scale(18)?;

    Ok(balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn reads_the_ledger_read_only() {
        let host = match env::var("TEST_DATABASE_HOST") {
            Ok(host) => host,
            Err(_) => return,
        };
        let bridge = Bridge::postgres(PostgresSettings {
            host,
            port: env::var("TEST_DATABASE_PORT").unwrap().parse().unwrap(),
            dbname: env::var("TEST_DATABASE_NAME").unwrap(),
            user: env::var("TEST_DATABASE_USER").unwrap(),
            password: env::var("TEST_DATABASE_PASSWORD").ok(),
        })
        .await
        .unwrap();
        assert_eq!(
            bridge.get_unwrapped_ban_balance().await.unwrap().to_string(),
            env::var("TEST_EXPECTED_UNWRAPPED").unwrap()
        );
        assert_eq!(
            bridge.count_stuck_withdrawals().await.unwrap().to_string(),
            env::var("TEST_EXPECTED_STUCK").unwrap()
        );
        if let Bridge::Postgres(client) = &bridge {
            let write = client.execute("delete from accounts", &[]).await;
            assert!(write.unwrap_err().to_string().contains("read-only"));
        }
    }
}
