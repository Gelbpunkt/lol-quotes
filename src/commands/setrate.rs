use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::CallbackDataBuilder;

use crate::Error;

use super::Context;

#[derive(CommandModel, CreateCommand, Debug, PartialEq, Eq)]
#[command(
    name = "setrate",
    desc = "Set the percentage of messages you want to be quoted on"
)]
pub struct SetrateCommand {
    #[command(
        desc = "Percentage of messages to quote you on",
        min_value = 0,
        max_value = 100
    )]
    pub percentage: i64,
}

impl SetrateCommand {
    pub async fn run(&self, context: Context) -> Result<(), Error> {
        let text = match context
            .database
            .set_rate(context.user_id, self.percentage)
            .await
        {
            Ok(_) => format!("Done. Your quote rate is now {}%", self.percentage),
            Err(_) => String::from("Failed to update your quote rate."),
        };

        let reply = CallbackDataBuilder::new()
            .content(text)
            .flags(MessageFlags::EPHEMERAL)
            .build();

        context
            .http
            .interaction_callback(
                context.interaction_id,
                &context.interaction_token,
                &InteractionResponse::ChannelMessageWithSource(reply),
            )
            .exec()
            .await?;

        Ok(())
    }
}
