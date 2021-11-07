mod banano;
mod bridge;
mod wban;
mod notifiers;

use crate::banano::Banano;
use crate::bridge::Bridge;
use crate::wban::WBan;
use crate::notifiers::{Notifier, TelegramNotifier};
use rust_decimal::Decimal;
use ethers::prelude::*;
use dotenv::dotenv;
use std::env;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() ->  Result<()> {
    dotenv().ok();
    let blockchain_network = env::var("BLOCKCHAIN_NETWORK").unwrap_or(String::from("BSC"));
    let banano_rpc_api = env::var("BAN_RPC_API").expect("Missing BAN_RPC_API env variable");
    let hot_wallet = env::var("BAN_HOT_WALLET").expect("Missing BAN_HOT_WALLET env variable");
    let cold_wallet = env::var("BAN_COLD_WALLET").expect("Missing BAN_COLD_WALLET env variable");
    let bc_rpc = env::var("BC_RPC").expect("Missing BC_RPC env variable");
    let redis_host = env::var("REDIS_HOST").expect("Missing REDIS_HOST env variable");

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

        let message = format!("We have a critical emergency on *{}* wBAN bridge:
A: Hot wallet  : `{}` BAN
B: Cold wallet : `{}` BAN
C: Unwrapped   : `{}` BAN
D: Total Supply: `{}` wBAN
\\-\\-\\-
Delta A\\+B\\-C\\-D  : `{}` BAN",
            &blockchain_network,
            hot_wallet_balance.to_string().as_str().replace(".", "\\."),
            cold_wallet_balance.to_string().as_str().replace(".", "\\."),
            unwrapped_balance.to_string().as_str().replace(".", "\\."),
            total_supply.to_string().as_str().replace(".", "\\."),
            delta.to_string().as_str().replace(".", "\\.")
        );

        println!("{}", message);
        /*
        eprintln!("(A) Hot wallet  : {:#?} BAN", hot_wallet_balance);
        eprintln!("(B) Cold wallet : {:#?} BAN", cold_wallet_balance);
        eprintln!("(C) Unwrapped   : {:#?} BAN", unwrapped_balance);
        eprintln!("(D) Total Supply: {:#?} wBAN", total_supply);
        eprintln!("---");
        eprintln!("Delta (A+B-C-D) : {:#?} BAN", delta);
        */
        let notifier: Box<dyn Notifier> = TelegramNotifier::new();
        notifier.alert_for_total_supply_error(&message).await.unwrap();
    }

    Ok(())
}
