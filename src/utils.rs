use sqlx::postgres::types::PgInterval;

pub fn parse_pg_interval(i: PgInterval) -> String {
    let ms = i.microseconds;

    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    let mut out = vec![];

    if years > 0 {
        out.push(format!("{} years", years));
    }

    if months > 0 {
        out.push(format!("{} months", months));
    }

    if weeks > 0 {
        out.push(format!("{} weeks", weeks));
    }

    if days > 0 {
        out.push(format!("{} days", days));
    }

    if hours > 0 {
        out.push(format!("{} hours", hours));
    }

    if minutes > 0 {
        out.push(format!("{} minutes", minutes));
    }

    if seconds > 0 {
        out.push(format!("{} seconds", seconds));
    }

    out.join(", ")
}