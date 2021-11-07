mod banano;
mod bridge;
mod wban;
mod reddit;

use crate::banano::Banano;
use crate::bridge::Bridge;
use crate::wban::WBan;
use crate::reddit::{Notifier, RedditNotifier};
use rust_decimal::Decimal;
use ethers::prelude::*;
use std::env;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() ->  Result<()> {
    let banano_rpc_api = env::var("BAN_RPC_API").expect("Missing BAN_RPC_API env variable");
    let hot_wallet = env::var("BAN_HOT_WALLET").expect("Missing BAN_HOT_WALLET env variable");
    let cold_wallet = env::var("BAN_COLD_WALLET").expect("Missing BAN_COLD_WALLET env variable");
    let bc_rpc = env::var("BC_RPC").expect("Missing BC_RPC env variable");
    let redis_host = env::var("REDIS_HOST").expect("Missing REDIS_HOST env variable");
    let users: Vec<String> = env::var("REDDIT_BOT_DM_USERS").expect("Missing REDDIT_BOT_DM_USERS env variable")
        .split_whitespace()
        .map(|user| String::from(user))
        .collect();

    let banano = Banano::new(banano_rpc_api);
    let hot_wallet_balance: Decimal = banano.get_banano_balance(&hot_wallet).await?;
    let cold_wallet_balance: Decimal = banano.get_banano_balance(&cold_wallet).await?;

    let bridge = Bridge::new(redis_host);
    let unwrapped_balance = bridge.get_unwrapped_ban_balance().await?;
    
    let wban = WBan::new(bc_rpc);
    let total_supply_raw: U256 = wban.fetch_total_supply().await.context("Can't fetch wBAN total supply")?;
    let mut total_supply = Decimal::from_str_radix(total_supply_raw.to_string().as_str(), 10)?;
    total_supply.set_scale(18)?;

    let mut total_users_deposits_balance: Decimal = hot_wallet_balance
        .checked_add(cold_wallet_balance).context("Overflow when adding hot and cold BAN balances")?
        .checked_sub(unwrapped_balance).context("Overflow")?;
    total_users_deposits_balance.set_scale(0).expect("Can't change total supply scale to 0");
    let total_users_deposits_balance: U256 = U256::from_dec_str(total_users_deposits_balance.to_string().as_str())?;
    let delta: Option<U256> = total_supply_raw.checked_sub(total_users_deposits_balance);

    if delta.is_some() {
        let mut delta = Decimal::from_str_radix(delta.unwrap().to_string().as_str(), 10)?;
        delta.set_scale(18)?;

        eprintln!("(A) Hot wallet  : {:#?} BAN", hot_wallet_balance);
        eprintln!("(B) Cold wallet : {:#?} BAN", cold_wallet_balance);
        eprintln!("(C) Unwrapped   : {:#?} BAN", unwrapped_balance);
        eprintln!("(D) Total Supply: {:#?} wBAN", total_supply);
        eprintln!("---");
        eprintln!("Delta (A+B-C-D) : {:#?} BAN", delta);
        let notifier: Box<dyn Notifier> = RedditNotifier::new(users);
        notifier.alert_for_total_supply_error().await?;
    }

    Ok(())
}
