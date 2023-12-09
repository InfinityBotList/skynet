use log::{error, info};
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::task::JoinSet;

#[derive(EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Task {
    UpdateStatus
}

pub async fn start_all_tasks(
    pool: sqlx::PgPool,
    cache_http: crate::cache::CacheHttpImpl,
    ctx: serenity::client::Context,
) -> ! {
    // Start tasks
    let mut set = JoinSet::new();

    for task in Task::iter() {
        set.spawn(crate::tasks::taskcat::taskcat(
            pool.clone(),
            cache_http.clone(),
            ctx.clone(),
            task,
        ));
    }

    if let Some(res) = set.join_next().await {
        if let Err(e) = res {
            error!("Error while running task: {}", e);
        }

        info!("Task finished when it shouldn't have");
        std::process::abort();
    }

    info!("All tasks finished when they shouldn't have");
    std::process::abort();
}

async fn taskcat(
    pool: sqlx::PgPool,
    cache_http: crate::cache::CacheHttpImpl,
    ctx: serenity::client::Context,
    task: Task,
) -> ! {
    let duration = match task {
        Task::UpdateStatus => Duration::from_secs(600),
    };

    let task_desc = match task {
        Task::UpdateStatus => "Update status",
    };

    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;

        log::info!(
            "TASK: {} ({}s interval) [{}]",
            task.to_string(),
            duration.as_secs(),
            task_desc
        );

        if let Err(e) = match task {
            Task::UpdateStatus => crate::tasks::update_status::update_status(&pool, &cache_http, &ctx).await,
        } {
            log::error!("TASK {} ERROR'd: {:?}", task.to_string(), e);
        }
    }
}
