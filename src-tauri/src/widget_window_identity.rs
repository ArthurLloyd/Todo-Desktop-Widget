use std::{collections::HashSet, sync::Mutex};

use uuid::Uuid;

use crate::models::WidgetInstanceDto;

const PRIMARY_WINDOW_LABEL: &str = "main";
const SECONDARY_LABEL_PREFIX: &str = "widget-";

pub fn widget_window_label(instance: &WidgetInstanceDto) -> Result<String, String> {
    if instance.is_primary {
        return Ok(PRIMARY_WINDOW_LABEL.to_string());
    }
    validate_instance_id(&instance.id)?;
    Ok(format!("{SECONDARY_LABEL_PREFIX}{}", instance.id))
}

pub fn widget_id_from_window_label(label: &str) -> Result<Option<String>, String> {
    if label == PRIMARY_WINDOW_LABEL {
        return Ok(None);
    }
    let id = label
        .strip_prefix(SECONDARY_LABEL_PREFIX)
        .ok_or_else(|| format!("not a widget window label: {label}"))?;
    validate_instance_id(id)?;
    Ok(Some(id.to_string()))
}

#[derive(Default)]
pub struct OpeningRegistry(Mutex<HashSet<String>>);

impl OpeningRegistry {
    pub fn begin(&self, label: &str) -> Result<bool, String> {
        let mut opening = self
            .0
            .lock()
            .map_err(|_| "widget opening state lock poisoned".to_string())?;
        Ok(opening.insert(label.to_string()))
    }

    pub fn finish(&self, label: &str) {
        if let Ok(mut opening) = self.0.lock() {
            opening.remove(label);
        }
    }
}

fn validate_instance_id(id: &str) -> Result<(), String> {
    let parsed = Uuid::parse_str(id).map_err(|_| "invalid widget instance id".to_string())?;
    if parsed.to_string() != id.to_ascii_lowercase() {
        return Err("widget instance id is not in canonical UUID form".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WidgetLayoutMode;

    #[test]
    fn secondary_window_label_round_trips() {
        let id = Uuid::new_v4().to_string();
        let instance = WidgetInstanceDto {
            id: id.clone(),
            name: "Secondary".to_string(),
            layout_mode: WidgetLayoutMode::Single,
            default_theme_id: None,
            is_primary: false,
            is_open: true,
            pinned: false,
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        let label = widget_window_label(&instance).unwrap();
        assert_eq!(widget_id_from_window_label(&label).unwrap(), Some(id));
        assert_eq!(widget_id_from_window_label("main").unwrap(), None);
    }

    #[test]
    fn invalid_widget_labels_are_rejected() {
        assert!(widget_id_from_window_label("widget-not-a-uuid").is_err());
        assert!(widget_id_from_window_label("settings").is_err());
    }

    #[test]
    fn duplicate_open_guard_allows_only_first_request() {
        let opening = OpeningRegistry::default();
        assert!(opening.begin("widget-test").unwrap());
        assert!(!opening.begin("widget-test").unwrap());
        opening.finish("widget-test");
        assert!(opening.begin("widget-test").unwrap());
    }
}
