#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayActivationAction {
    ActivateNormal,
    ShowAndEnforcePinned,
}

pub fn action_for_pinned(pinned: bool) -> TrayActivationAction {
    if pinned {
        TrayActivationAction::ShowAndEnforcePinned
    } else {
        TrayActivationAction::ActivateNormal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_activation_distinguishes_normal_and_pinned_windows() {
        assert_eq!(
            action_for_pinned(false),
            TrayActivationAction::ActivateNormal
        );
        assert_eq!(
            action_for_pinned(true),
            TrayActivationAction::ShowAndEnforcePinned
        );
    }
}
