#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationProgress {
    pub title: String,
    pub phase: String,
    pub item: Option<String>,
    pub completed: Option<usize>,
    pub total: Option<usize>,
    pub indeterminate: bool,
    pub logs: Vec<String>,
}

impl OperationProgress {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            phase: "Starting".to_owned(),
            item: None,
            completed: None,
            total: None,
            indeterminate: true,
            logs: Vec::new(),
        }
    }

    pub fn with_counts(mut self, completed: usize, total: usize) -> Self {
        self.completed = Some(completed);
        self.total = Some(total);
        self.indeterminate = total == 0;
        self
    }

    pub fn percent(&self) -> Option<f32> {
        let total = self.total?;
        if total == 0 {
            return None;
        }
        Some(self.completed.unwrap_or_default() as f32 / total as f32)
    }

    pub fn bar_fraction(&self) -> Option<f32> {
        if self.indeterminate {
            return None;
        }
        self.percent()
    }

    pub fn apply(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::Phase {
                phase,
                item,
                completed,
                total,
                indeterminate,
            } => {
                self.phase = phase;
                self.item = item;
                self.completed = completed;
                self.total = total;
                self.indeterminate = indeterminate;
            }
            ProgressEvent::Log(line) => {
                self.logs.push(line);
                if self.logs.len() > 8 {
                    let overflow = self.logs.len() - 8;
                    self.logs.drain(0..overflow);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgressEvent {
    Phase {
        phase: String,
        item: Option<String>,
        completed: Option<usize>,
        total: Option<usize>,
        indeterminate: bool,
    },
    Log(String),
}

impl ProgressEvent {
    pub fn phase(phase: impl Into<String>) -> Self {
        Self::Phase {
            phase: phase.into(),
            item: None,
            completed: None,
            total: None,
            indeterminate: true,
        }
    }

    pub fn phase_item(
        phase: impl Into<String>,
        item: impl Into<String>,
        completed: usize,
        total: usize,
    ) -> Self {
        Self::Phase {
            phase: phase.into(),
            item: Some(item.into()),
            completed: Some(completed),
            total: Some(total),
            indeterminate: total == 0,
        }
    }

    pub fn log(line: impl Into<String>) -> Self {
        Self::Log(line.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_is_known_when_total_is_nonzero() {
        let progress = OperationProgress::new("Backup").with_counts(2, 4);
        assert_eq!(progress.percent(), Some(0.5));
    }

    #[test]
    fn progress_event_updates_phase_and_item() {
        let mut progress = OperationProgress::new("Applet flash");
        progress.apply(ProgressEvent::phase_item("installing", "AlphaWord", 1, 3));
        assert_eq!(progress.phase, "installing");
        assert_eq!(progress.item.as_deref(), Some("AlphaWord"));
        assert_eq!(progress.completed, Some(1));
        assert_eq!(progress.total, Some(3));
    }

    #[test]
    fn indeterminate_progress_has_no_bar_fraction() {
        let progress = OperationProgress::new("Flash");

        assert_eq!(progress.bar_fraction(), None);
    }
}
