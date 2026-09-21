use crate::models::{SettingsDto, SettingsPatch};

pub fn apply_patch(mut settings: SettingsDto, patch: SettingsPatch) -> SettingsDto {
    if let Some(value) = patch.always_on_top {
        settings.always_on_top = value;
    }

    if let Some(value) = patch.opacity {
        if value.is_finite() {
            settings.opacity = value.clamp(0.0, 1.0);
        }
    }

    if let Some(value) = patch.sync_interval_minutes {
        settings.sync_interval_minutes = value.clamp(1, 60);
    }

    if let Some(value) = patch.autostart {
        settings.autostart = value;
    }

    if let Some(value) = patch.debug_mode {
        settings.debug_mode = value;
    }

    if let Some(value) = patch.show_completed {
        settings.show_completed = value;
    }

    if let Some(value) = patch.language {
        settings.language = match value.as_str() {
            "en" | "zh-CN" => value,
            _ => "system".to_string(),
        };
    }

    if let Some(value) = patch.font_family {
        settings.font_family = match value.as_str() {
            "compact" | "serif" | "mono" => value,
            _ => "system".to_string(),
        };
    }

    if let Some(value) = patch.font_scale {
        settings.font_scale = value.clamp(0.9, 1.2);
    }

    settings
}

pub fn sanitize(mut settings: SettingsDto) -> SettingsDto {
    settings.opacity = if settings.opacity.is_finite() {
        settings.opacity.clamp(0.0, 1.0)
    } else {
        SettingsDto::default().opacity
    };
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opacity_patch(opacity: f64) -> SettingsPatch {
        SettingsPatch {
            always_on_top: None,
            opacity: Some(opacity),
            sync_interval_minutes: None,
            autostart: None,
            debug_mode: None,
            show_completed: None,
            language: None,
            font_family: None,
            font_scale: None,
        }
    }

    #[test]
    fn opacity_accepts_the_full_zero_to_one_range() {
        for value in [0.0, 0.01, 0.5, 1.0] {
            assert_eq!(
                apply_patch(SettingsDto::default(), opacity_patch(value)).opacity,
                value
            );
        }
    }

    #[test]
    fn opacity_clamps_finite_out_of_range_values() {
        assert_eq!(
            apply_patch(SettingsDto::default(), opacity_patch(-0.01)).opacity,
            0.0
        );
        assert_eq!(
            apply_patch(SettingsDto::default(), opacity_patch(1.01)).opacity,
            1.0
        );
    }

    #[test]
    fn opacity_ignores_non_finite_patches_and_sanitizes_loaded_values() {
        let current = SettingsDto::default();
        assert_eq!(
            apply_patch(current.clone(), opacity_patch(f64::NAN)).opacity,
            current.opacity
        );
        assert_eq!(
            apply_patch(current.clone(), opacity_patch(f64::INFINITY)).opacity,
            current.opacity
        );
        let mut loaded = current;
        loaded.opacity = f64::NAN;
        assert_eq!(sanitize(loaded).opacity, SettingsDto::default().opacity);
    }
}
