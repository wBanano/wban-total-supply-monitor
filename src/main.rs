mod banano;
mod wban;
mod reddit;

use web3::types::U256;
use crate::banano::Banano;
use crate::wban::WBan;
use crate::reddit::{Notifier, RedditNotifier};
use error_chain::error_chain;
use rust_decimal::Decimal;
use std::env;

error_chain! {
    foreign_links {
        EnvVar(env::VarError);
        HttpRequest(reqwest::Error);
    }
}

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
        .checked_add(cold_wallet_balance)
        .unwrap();
    
    let wban = WBan::new();
    let total_supply: U256 = wban.fetch_total_supply().await.unwrap();

    total_users_deposits_balance.set_scale(0).unwrap();
    let total_users_deposits_balance: U256 = U256::from_dec_str(total_users_deposits_balance.to_string().as_str()).unwrap();
    let delta: Option<U256> = total_supply.checked_sub(total_users_deposits_balance);

    if delta.is_some() {
        println!("Red flag!!! Delta: {:#?}", delta.unwrap());
        let notifier: Box<dyn Notifier> = RedditNotifier::new(users);
        notifier.alert_for_total_supply_error().await?;
    }

    Ok(())
}
