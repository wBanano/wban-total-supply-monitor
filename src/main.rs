mod banano;
mod wban;
mod reddit;

use web3::types::U256;
use crate::banano::Banano;
use crate::wban::WBan;
use crate::reddit::{Notifier, RedditNotifier};
use rust_decimal::Decimal;
use std::env;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() ->  Result<()> {
    let banano_rpc_api = env::var("BAN_RPC_API").expect("Missing BAN_RPC_API env variable");
    let hot_wallet = env::var("BAN_HOT_WALLET").expect("Missing BAN_HOT_WALLET env variable");
    let cold_wallet = env::var("BAN_COLD_WALLET").expect("Missing BAN_COLD_WALLET env variable");
    let users: Vec<String> = env::var("REDDIT_BOT_DM_USERS").expect("Missing REDDIT_BOT_DM_USERS env variable")
        .split_whitespace()
        .map(|user| String::from(user))
        .collect();

    let banano = Banano::new(banano_rpc_api);
    let hot_wallet_balance: Decimal = banano.get_banano_balance(&hot_wallet).await?;
    let cold_wallet_balance: Decimal = banano.get_banano_balance(&cold_wallet).await?;
    let mut total_users_deposits_balance: Decimal = hot_wallet_balance
        .checked_add(cold_wallet_balance).context("Overflow when adding hot and cold BAN balances")?;
    
    let wban = WBan::new();
    let total_supply: U256 = wban.fetch_total_supply().await.context("Can't fetch wBAN total supply")?;

    total_users_deposits_balance.set_scale(0).expect("Can't change total supply scale to 0");
    let total_users_deposits_balance: U256 = U256::from_dec_str(total_users_deposits_balance.to_string().as_str())?;
    let delta: Option<U256> = total_supply.checked_sub(total_users_deposits_balance);

    if delta.is_some() {
        eprintln!("Red flag!!! Delta: {:#?}", delta.unwrap());
        let notifier: Box<dyn Notifier> = RedditNotifier::new(users);
        notifier.alert_for_total_supply_error().await?;
    }

    Ok(())
}
