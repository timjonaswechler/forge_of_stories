use crate::{
    action::{
        Action, LogicAction, Notification, NotificationLevel, TaskId, TaskProgress, TaskResult,
        UiAction,
    },
    components::Component,
};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, sleep};

/// Task manager + list: tracks task progress and emits notifications on completion.
pub(crate) struct TaskList {
    tx: Option<UnboundedSender<Action>>,
    tasks: HashMap<TaskId, TaskInfo>,
    focused: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TaskInfo {
    label: String,
    progress: Option<f32>,
    message: Option<String>,
    success: Option<bool>,
}

impl TaskList {
    pub(crate) fn new() -> Self {
        Self {
            tx: None,
            tasks: HashMap::new(),
            focused: false,
        }
    }

    /// Example: spawn a demo background task (scaffolding).
    /// Emits TaskStarted, periodic TaskProgress, and TaskCompleted via action channel.
    #[allow(dead_code)]
    pub(crate) fn spawn_demo_task(&self, id: TaskId, label: String) {
        if let Some(tx) = &self.tx {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));
                for i in 1..=10 {
                    sleep(Duration::from_millis(200)).await;
                    let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                        id: id.clone(),
                        fraction: Some(i as f32 / 10.0),
                        message: Some(format!("{} — step {}/10", label, i)),
                    })));
                }
                let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                    id: id.clone(),
                    success: true,
                    result_json: None,
                    message: Some(format!("{} — done", label)),
                })));
            });
        }
    }
}

impl Component for TaskList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Logic(LogicAction::TaskStarted { id }) => {
                // Ensure a task entry exists
                self.tasks.entry(id).or_insert_with(|| TaskInfo {
                    label: "Task".to_string(),
                    ..Default::default()
                });
            }
            Action::Logic(LogicAction::TaskProgress(TaskProgress {
                id,
                fraction,
                message,
            })) => {
                let entry = self.tasks.entry(id).or_insert_with(Default::default);
                entry.progress = fraction;
                if let Some(m) = message {
                    entry.message = Some(m);
                }
            }
            Action::Logic(LogicAction::TaskCompleted(TaskResult {
                id,
                success,
                result_json: _,
                message,
            })) => {
                let entry = self
                    .tasks
                    .entry(id.clone())
                    .or_insert_with(Default::default);
                entry.success = Some(success);
                entry.message = message.clone();

                // Emit a notification on completion
                if let Some(tx) = &self.tx {
                    let level = if success {
                        NotificationLevel::Success
                    } else {
                        NotificationLevel::Error
                    };
                    let msg = message.unwrap_or_else(|| "Task completed".to_string());
                    let _ = tx.send(Action::Ui(UiAction::ShowNotification(Notification {
                        id: format!("task-{}", id),
                        level,
                        message: msg,
                        timeout_ms: None, // App will apply default lifetime
                    })));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Render a simple list of tasks with progress and status
        let mut lines: Vec<String> = Vec::new();
        for (id, t) in self.tasks.iter() {
            let pct = t
                .progress
                .map(|p| format!("{:>3}%", (p * 100.0) as u32))
                .unwrap_or("--%".to_string());
            let status = match t.success {
                Some(true) => "OK",
                Some(false) => "ERR",
                None => "RUN",
            };
            let msg = t.message.as_deref().unwrap_or(if t.success.is_some() {
                ""
            } else {
                "working..."
            });
            lines.push(format!("[{}] {} {} {}", status, pct, id, msg));
        }
        if lines.is_empty() {
            lines.push("No running tasks".to_string());
        }
        let mut block = Block::default().borders(Borders::ALL).title("Tasks");
        if self.focused {
            block = block.style(Style::default().fg(Color::Yellow));
        }
        let para = Paragraph::new(lines.join("\n")).block(block);
        f.render_widget(para, area);
        Ok(())
    }
}
