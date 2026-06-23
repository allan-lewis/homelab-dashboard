use gloo_net::http::Request;

use crate::frontend::models::TaskStatus;

pub async fn fetch_tasks() -> Vec<TaskStatus> {
    let mut tasks = match Request::get("/api/tasks").send().await {
        Ok(response) => response.json::<Vec<TaskStatus>>().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    tasks.sort_by(|a, b| {
        a.instance
            .cmp(&b.instance)
            .then_with(|| a.name.cmp(&b.name))
    });

    tasks
}

pub fn task_is_late(task: &TaskStatus) -> bool {
    // task.age_ratio >= 1.05
    task.age_ratio >= 0.85
}

pub fn task_status_lines(tasks: &[TaskStatus]) -> Vec<String> {
    let late_count = tasks
        .iter()
        .filter(|task| task_is_late(task))
        .count();

    if late_count == 0 {
        Vec::new()
    } else {
        vec![format!("{late_count} scheduled tasks are late.")]
    }
}
