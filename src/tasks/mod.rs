pub mod update_status;

use botox::taskman::Task;
use futures_util::FutureExt;

pub fn tasks() -> Vec<Task> {
    vec![
        Task {
            name: "Update Status",
            description: "Update the status of the bot",
            enabled: true,
            duration: std::time::Duration::from_secs(600),
            run: Box::new(move |ctx| {
                update_status::update_status(ctx).boxed()
            })
        }
    ]
}