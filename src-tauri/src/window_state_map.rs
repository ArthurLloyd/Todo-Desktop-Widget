use std::collections::HashMap;

struct PerWindowState<T> {
    attachment: Option<T>,
    pinned: bool,
}

impl<T> Default for PerWindowState<T> {
    fn default() -> Self {
        Self {
            attachment: None,
            pinned: false,
        }
    }
}

pub struct WindowStateMap<T> {
    windows: HashMap<String, PerWindowState<T>>,
}

impl<T> Default for WindowStateMap<T> {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }
}

impl<T> WindowStateMap<T> {
    pub fn is_attached(&self, label: &str) -> bool {
        self.windows
            .get(label)
            .is_some_and(|state| state.attachment.is_some())
    }

    pub fn pinned(&self, label: &str) -> bool {
        self.windows.get(label).is_some_and(|state| state.pinned)
    }

    pub fn set_pinned(&mut self, label: &str, pinned: bool) {
        self.windows.entry(label.to_string()).or_default().pinned = pinned;
    }

    pub fn set_attachment(&mut self, label: &str, attachment: Option<T>) {
        self.windows
            .entry(label.to_string())
            .or_default()
            .attachment = attachment;
    }

    pub fn remove(&mut self, label: &str) {
        self.windows.remove(label);
    }
}

impl<T: Clone> WindowStateMap<T> {
    pub fn attachment(&self, label: &str) -> Option<T> {
        self.windows
            .get(label)
            .and_then(|state| state.attachment.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_window_pinned_state_is_independent() {
        let mut windows = WindowStateMap::<()>::default();
        windows.set_pinned("main", true);
        windows.set_pinned("widget-b", false);
        assert!(windows.pinned("main"));
        assert!(!windows.pinned("widget-b"));

        windows.set_pinned("widget-b", true);
        assert!(windows.pinned("main"));
        assert!(windows.pinned("widget-b"));
    }
}
