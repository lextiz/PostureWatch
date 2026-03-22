pub enum AlertEvent {
    None,
    FirstWarning,
    NotifyBadPosture,
    PostureImproved,
}

pub struct MonitorLogic {
    pub consecutive_bad: u32,
}

impl MonitorLogic {
    pub fn new() -> Self {
        Self { consecutive_bad: 0 }
    }

    pub fn process_status(&mut self, status: super::posture::PostureStatus) -> AlertEvent {
        match status {
            super::posture::PostureStatus::Bad => {
                self.consecutive_bad += 1;
                if self.consecutive_bad >= 2 {
                    AlertEvent::NotifyBadPosture
                } else {
                    AlertEvent::FirstWarning
                }
            }
            super::posture::PostureStatus::Good => {
                if self.consecutive_bad > 0 {
                    self.consecutive_bad = 0;
                    AlertEvent::PostureImproved
                } else {
                    self.consecutive_bad = 0;
                    AlertEvent::None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posture::PostureStatus;

    #[test]
    fn test_alert_repeat_behavior() {
        let mut logic = MonitorLogic::new();

        // Initial good
        assert!(matches!(
            logic.process_status(PostureStatus::Good),
            AlertEvent::None
        ));

        // First bad - warning only
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::FirstWarning
        ));

        // Second bad - notify
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));

        // Third bad - notify again (repeat until improves)
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));

        // Improves
        assert!(matches!(
            logic.process_status(PostureStatus::Good),
            AlertEvent::PostureImproved
        ));

        // Good again
        assert!(matches!(
            logic.process_status(PostureStatus::Good),
            AlertEvent::None
        ));
    }
}
