pub use serenity::client::Context;
pub use serenity::framework::standard::{Args, Command, CommandError};
pub use serenity::model::channel::Message;

pub type CommandResult = Result<(), CommandError>;
