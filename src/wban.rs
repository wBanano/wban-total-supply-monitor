use web3::{
    contract::{Contract, Options},
    types::{Address, U256},
};
use thiserror::Error;
use std::str::FromStr;

pub struct WBan {
    wban_address: Address,
}

#[derive(Error, Debug)]
pub enum WBANError {
    #[error("Web3 error")]
    Web3(#[from] web3::Error),
    #[error("Web3 ETH ABI error")]
    Web3EthABI(#[from] web3::ethabi::Error),
    #[error("Web3 contract error")]
    Web3Contract(#[from] web3::contract::Error),
}

impl WBan {
    pub fn new() -> WBan {
        WBan {
            wban_address: Address::from_str("e20b9e246db5a0d21bf9209e4858bc9a3ff7a034").unwrap(),
        }
    }

    pub async fn fetch_total_supply(&self) -> Result<U256, WBANError> {
        let transport = web3::transports::Http::new("https://bsc-dataseed.binance.org/")?;
        let web3 = web3::Web3::new(transport);

        let contract = Contract::from_json(web3.eth(), self.wban_address, include_bytes!("../res/WBANToken.json"))?;
        let result = contract.query("totalSupply", (), None, Options::default(), None);
        let total_supply: U256 = result.await?;

        Ok(total_supply)
    }
}
