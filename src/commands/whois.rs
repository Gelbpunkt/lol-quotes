use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::callback::InteractionResponse, channel::message::MessageFlags, user::User,
};
use twilight_util::builder::CallbackDataBuilder;

use crate::Error;

use super::Context;

#[derive(CommandModel, CreateCommand, Debug, PartialEq, Eq)]
#[command(
    name = "whois",
    desc = "View who someone set as their League of Legends champion"
)]
pub struct WhoisCommand {
    #[command(desc = "The person whose champion you want to see")]
    pub user: User,
}

impl WhoisCommand {
    pub async fn run(&self, context: Context) -> Result<(), Error> {
        let text = match context
            .database
            .get_champion_and_rate(self.user.id.get() as i64)
            .await
        {
            Ok((champion, _)) => champion,
            Err(_) => String::from("Failed to fetch their champion."),
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
