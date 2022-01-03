use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::callback::InteractionResponse, channel::message::MessageFlags};
use twilight_util::builder::CallbackDataBuilder;

use crate::{champions::ALL_CHAMPIONS_STRING, Error};

use super::Context;

#[derive(CommandModel, CreateCommand, Debug, PartialEq, Eq)]
#[command(
    name = "champions",
    desc = "View all available League of Legends champions"
)]
pub struct ChampionsCommand {}

impl ChampionsCommand {
    pub async fn run(&self, context: Context) -> Result<(), Error> {
        let reply = CallbackDataBuilder::new()
            .content(ALL_CHAMPIONS_STRING.clone())
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
