use dashmap::DashMap;
use twilight_http::Client;
use twilight_model::{channel::Webhook, id::ChannelId};

use std::sync::Arc;

use crate::error::Error;

pub struct Webhooks {
    client: Arc<Client>,
    cache: DashMap<ChannelId, Webhook>,
}

impl Webhooks {
    pub fn with_client(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: DashMap::new(),
        }
    }

    async fn create_new(&self, channel_id: ChannelId) -> Result<Webhook, Error> {
        let webhook = self
            .client
            .create_webhook(channel_id, "LolQuotes")
            .exec()
            .await?
            .model()
            .await?;

        self.cache.insert(channel_id, webhook.clone());

        Ok(webhook)
    }

    async fn fetch_best_in_channel(&self, channel_id: ChannelId) -> Result<Option<Webhook>, Error> {
        let webhooks = self
            .client
            .channel_webhooks(channel_id)
            .exec()
            .await?
            .models()
            .await?;

        let webhook = webhooks.into_iter().find(|hook| hook.token.is_some());

        if let Some(webhook) = webhook {
            self.cache.insert(channel_id, webhook.clone());
            Ok(Some(webhook))
        } else {
            Ok(None)
        }
    }

    pub async fn get_webhook_for_channel(&self, channel_id: ChannelId) -> Result<Webhook, Error> {
        if let Some(webhook) = self.cache.get(&channel_id) {
            Ok(webhook.clone())
        } else {
            if let Some(webhook) = self.fetch_best_in_channel(channel_id).await? {
                Ok(webhook)
            } else {
                self.create_new(channel_id).await
            }
        }
    }
}
