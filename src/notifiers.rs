use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use async_trait::async_trait;
use teloxide::types::ParseMode;
use std::env;
use std::error::Error;

#[async_trait]
pub trait Notifier {
    async fn alert_for_total_supply_error(&self, message: &String) -> Result<(), Box<dyn Error>> ;
}

pub struct TelegramNotifier {
    bot: AutoSend<Bot>,
}

impl TelegramNotifier {
    pub fn new() -> Box<dyn Notifier> {
        teloxide::enable_logging!();
        Box::new(TelegramNotifier {
            bot: Bot::from_env().auto_send(),
        })
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn alert_for_total_supply_error(&self, message: &String) -> Result<(), Box<dyn Error>> {
        self.bot.send_message(-572662736, message)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}
