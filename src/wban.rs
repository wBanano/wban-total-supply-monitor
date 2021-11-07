use ethers::{
    prelude::*,
    core::k256::ecdsa::SigningKey,
};
use std::{convert::TryFrom};
use thiserror::Error;
use std::str::FromStr;

pub struct WBan {
    wban_address: Address,
    bc_rpc: String,
}

#[derive(Error, Debug)]
pub enum WBANError {
    #[error("Web3 provider error")]
    ProviderError(#[from] ContractError<ethers::prelude::Provider<ethers::prelude::Http>>),
    #[error("Web3 contract error")]
    ContractError(#[from] ContractError<ethers::prelude::SignerMiddleware<ethers::prelude::Provider<ethers::prelude::Http>, Wallet<SigningKey>>>),
}

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    WBANToken,
    "./res/WBANToken.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

impl WBan {
    pub fn new(bc_rpc: String) -> WBan {
        WBan {
            wban_address: Address::from_str("e20b9e246db5a0d21bf9209e4858bc9a3ff7a034").unwrap(),
            bc_rpc,
        }
    }

    /// Fetch total supply of wBAN minted
    pub async fn fetch_total_supply(&self) -> Result<U256, WBANError> {
        let provider = Provider::<Http>::try_from(self.bc_rpc.clone())
            .expect("Invalid Web3 provider URL");

        let client = std::sync::Arc::new(provider);

        let contract = WBANToken::new(self.wban_address, client.clone());
        let total_supply = contract.total_supply().call().await?;

        //print!("Total supply: {}", total_supply);

        Ok(total_supply)
    }

    /*
    /// Pause wBAN smart-contract
    ///
    /// Only do this for emergency reasons!
    pub async fn pause(&self) -> Result<(), WBANError> {
        let provider = Provider::<Http>::try_from("https://bsc-dataseed.binance.org/")
            .expect("Invalid Web3 provider URL");

        let wallet = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
            .parse::<LocalWallet>()
            .expect("Can't parse wallet address");

        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        let contract = WBANToken::new(self.wban_address, client.clone());
        let _receipt = contract.pause().send().await?.await.unwrap();

        Ok(())
    }
    */
}
