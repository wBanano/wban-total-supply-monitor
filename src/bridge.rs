use redis::{Client, Commands, IntoConnectionInfo, RedisError};
use rust_decimal::Decimal;
use thiserror::Error;

pub struct Bridge {
    client: Client,
}

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Decimal error")]
    Decimal(#[from] rust_decimal::Error),
}

impl Bridge {

    pub fn new(redis_host: String, redis_password: Option<String>) -> Bridge {
        let mut connection_info = redis_host.into_connection_info().unwrap();
        if let Some(password) = redis_password {
            connection_info.redis.password = Some(password);
        }
        let client = Client::open(connection_info).unwrap();
        Bridge {
            client,
        }
    }

    pub async fn get_unwrapped_ban_balance(&self) -> Result<Decimal, RedisError> {
        let mut con = self.client.get_connection()?;
        let balances_keys: Vec<String> = con.keys("ban-balance:*")?;
        let unwrapped: Decimal = balances_keys
            .into_iter()
            // map Redis key to BAN balance with 18 digits as integer
            .map(|key| con.get(key).unwrap())
            // map BAN balance to Decimal with 18 digits
            .map(|raw| self.convert_raw_balance(raw).unwrap())
            .sum();

        Ok(unwrapped)
    }

    fn convert_raw_balance(&self, raw_balance: String) -> Result<Decimal, BridgeError> {
        if raw_balance == "0" /*|| raw_balance.len() <= 11*/ {
            return Ok(Decimal::from(0))
        }
        let mut balance: Decimal = Decimal::from_str_radix(raw_balance.as_str(), 10).unwrap();
        balance.set_scale(18)?;

        Ok(balance)
    }

}
