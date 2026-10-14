mod banano;
mod bridge;
mod wban;
mod notifiers;

use crate::banano::Banano;
use crate::bridge::{Bridge, PostgresSettings};
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

    let banano = Banano::new(banano_rpc_api);
    let hot_wallet_balance: Decimal = banano.get_banano_balance(&hot_wallet).await?;
    let cold_wallet_balance: Decimal = banano.get_banano_balance_with_pending(&cold_wallet).await?;

    let bridge = match env::var("DATABASE_HOST").ok().filter(|host| !host.is_empty()) {
        Some(host) => Bridge::postgres(PostgresSettings {
            host,
            port: env::var("DATABASE_PORT").ok().and_then(|port| port.parse().ok()).unwrap_or(5432),
            dbname: env::var("DATABASE_NAME").expect("Missing DATABASE_NAME env variable"),
            user: env::var("DATABASE_USER").expect("Missing DATABASE_USER env variable"),
            password: env::var("DATABASE_PASSWORD").ok().filter(|password| !password.is_empty()),
        }).await?,
        None => Bridge::redis(
            env::var("REDIS_HOST").expect("Missing REDIS_HOST env variable"),
            env::var("REDIS_PASSWORD").ok().filter(|password| !password.is_empty()),
        ),
    };
    let unwrapped_balance = bridge.get_unwrapped_ban_balance().await?;
    
    let wban = WBan::new(bc_rpc);
    let total_supply_raw: U256 = wban.fetch_total_supply().await.context("Can't fetch wBAN total supply")?;
    let mut total_supply = Decimal::from_str_radix(total_supply_raw.to_string().as_str(), 10)?;
    total_supply.set_scale(18)?;
    println!(
        "{}: hot wallet {} BAN, cold wallet {} BAN, unwrapped {} BAN from {}, total supply {} wBAN",
        &blockchain_network,
        hot_wallet_balance,
        cold_wallet_balance,
        unwrapped_balance,
        bridge.source(),
        total_supply
    );

    let mut total_users_deposits_balance: Decimal = hot_wallet_balance
        .checked_add(cold_wallet_balance).context("Overflow when adding hot and cold BAN balances")?
        .checked_sub(unwrapped_balance).context("Overflow when substracting unwrapped BAN balances")?;
    total_users_deposits_balance.set_scale(0).expect("Can't change total supply scale to 0");
    let total_users_deposits_balance: U256 = U256::from_dec_str(total_users_deposits_balance.to_string().as_str())?;
    let delta: Option<U256> = total_supply_raw.checked_sub(total_users_deposits_balance);

    if delta.is_some() && !delta.unwrap().is_zero() {
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
        eprintln!("(A) Hot wallet  : {:#?} BAN", hot_wallet_balance);
        eprintln!("(B) Cold wallet : {:#?} BAN", cold_wallet_balance);
        eprintln!("(C) Unwrapped   : {:#?} BAN", unwrapped_balance);
        eprintln!("(D) Total Supply: {:#?} wBAN", total_supply);
        eprintln!("---");
        eprintln!("Delta (A+B-C-D) : {:#?} BAN", delta);
        let notifier: Box<dyn Notifier> = TelegramNotifier::new();
        notifier.alert(&message).await.unwrap();
    }

    let stuck_withdrawals = bridge.count_stuck_withdrawals().await?;
    if stuck_withdrawals > 0 {
        let message = format!("*{}* wBAN bridge has {} withdrawal\\(s\\) reserved for more than 10 minutes\\. Resolve them with `yarn ledger:withdrawal`\\.",
            &blockchain_network,
            stuck_withdrawals
        );
        println!("{}", message);
        TelegramNotifier::new().alert(&message).await.unwrap();
    }

    Ok(())
}
