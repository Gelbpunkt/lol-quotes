use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::CallbackDataBuilder;

use crate::Error;

use super::Context;

#[derive(CommandModel, CreateCommand, Debug, PartialEq, Eq)]
#[command(
    name = "whoami",
    desc = "View who you set as their League of Legends champion"
)]
pub struct WhoamiCommand {}

impl WhoamiCommand {
    pub async fn run(&self, context: Context) -> Result<(), Error> {
        let text = match context
            .database
            .get_champion_and_rate(context.user_id)
            .await
        {
            Ok((champion, _)) => champion,
            Err(_) => String::from("Failed to fetch your champion."),
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
