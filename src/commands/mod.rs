pub mod champions;
pub mod iam;
pub mod setrate;
pub mod whoami;
pub mod whois;

use std::sync::Arc;

pub use champions::ChampionsCommand;
pub use iam::IamCommand;
pub use setrate::SetrateCommand;
use twilight_http::Client;
use twilight_model::id::InteractionId;
pub use whoami::WhoamiCommand;
pub use whois::WhoisCommand;

use crate::db::Database;

pub struct Context {
    pub http: Arc<Client>,
    pub database: Arc<Database>,
    pub user_id: i64,
    pub interaction_id: InteractionId,
    pub interaction_token: String,
}
