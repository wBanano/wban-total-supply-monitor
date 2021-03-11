use web3::{
    contract::{Contract, Options},
    types::{Address, U256},
};
use std::str::FromStr;

pub struct WBan {
    wban_address: Address,
}

impl WBan {
    pub fn new() -> WBan {
        WBan {
            wban_address: Address::from_str("e20b9e246db5a0d21bf9209e4858bc9a3ff7a034").unwrap(),
        }
    }

    pub async fn fetch_total_supply(&self) -> Result<U256, web3::Error> {
        let transport = web3::transports::Http::new("https://bsc-dataseed.binance.org/")?;
        let web3 = web3::Web3::new(transport);

        let contract = Contract::from_json(web3.eth(), self.wban_address, include_bytes!("../res/WBANToken.json")).unwrap();
        let result = contract.query("totalSupply", (), None, Options::default(), None);
        let total_supply: U256 = result.await.unwrap();

        Ok(total_supply)
    }
}