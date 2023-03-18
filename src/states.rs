use strum_macros::{Display, EnumString};

#[derive(PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OnboardState {
    Pending,
    QueueRemind,
    QueueForceClaim,
    Claimed,
    PendingManagerReview,
    Denied,
    Completed,
}