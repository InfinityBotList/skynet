use rand::seq::SliceRandom;
use serenity::{all::OnlineStatus, gateway::ActivityData};

enum Status {
    Watch,
    Play,
    Listen,
}

pub async fn update_status(
    ctx: &serenity::all::Context,
) -> Result<(), crate::Error> {
    let statuses = [
        (Status::Watch, "sky!help"),
        (Status::Play, "stopping raids"),
        (Status::Listen, "to your commands"),
    ];

    // Get random status
    let (status, text) = statuses.choose(&mut rand::thread_rng()).unwrap();

    let activity = match status {
        Status::Watch => Some(ActivityData::watching(text.to_string())),
        Status::Play => Some(ActivityData::playing(text.to_string())),
        Status::Listen => Some(ActivityData::listening(text.to_string())),
    };

    ctx.set_presence(activity, OnlineStatus::Online);

    Ok(())
}