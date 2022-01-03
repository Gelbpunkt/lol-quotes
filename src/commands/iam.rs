use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::{
        callback::{Autocomplete, InteractionResponse},
        command::CommandOptionChoice,
    },
    channel::message::MessageFlags,
};
use twilight_util::builder::CallbackDataBuilder;

use crate::{champions::CHAMPIONS, Error};

use super::Context;

#[derive(CommandModel, CreateCommand, Debug, PartialEq, Eq)]
#[command(
    name = "iam",
    desc = "Set a League of Legends champion to be quoted as"
)]
pub struct IamCommand {
    #[command(autocomplete = true, desc = "The champion you want to be quoted as")]
    pub champion: String,
}

impl IamCommand {
    pub async fn run(&self, context: Context) -> Result<(), Error> {
        let is_valid = CHAMPIONS.contains_key(&self.champion);

        let reply = if is_valid {
            match context
                .database
                .set_champion(context.user_id, &self.champion)
                .await
            {
                Ok(_) => CallbackDataBuilder::new()
                    .content(format!("You are now {}.", self.champion))
                    .flags(MessageFlags::EPHEMERAL)
                    .build(),
                Err(_) => CallbackDataBuilder::new()
                    .content(String::from("Failed to update your champion."))
                    .flags(MessageFlags::EPHEMERAL)
                    .build(),
            }
        } else {
            CallbackDataBuilder::new()
                .content(String::from("Invalid champion."))
                .flags(MessageFlags::EPHEMERAL)
                .build()
        };

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

    pub async fn autocomplete(&self, context: Context) -> Result<(), Error> {
        let matches: Vec<CommandOptionChoice> = CHAMPIONS
            .keys()
            .filter_map(|name| {
                if name.starts_with(&self.champion) {
                    Some(CommandOptionChoice::String {
                        name: name.to_string(),
                        value: name.to_string(),
                    })
                } else {
                    None
                }
            })
            .take(25)
            .collect();

        context
            .http
            .interaction_callback(
                context.interaction_id,
                &context.interaction_token,
                &InteractionResponse::Autocomplete(Autocomplete { choices: matches }),
            )
            .exec()
            .await?;

        Ok(())
    }
}
