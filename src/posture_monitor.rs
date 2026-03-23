pub enum AlertEvent {
    None,
    FirstWarning,
    NotifyBadPosture,
    PostureImproved,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Strictness {
    Low,    // Require 3 bad before alert
    Medium, // Require 2 bad before alert
    High,   // Alert on first bad (1)
}

impl Strictness {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Strictness::Low,
            "high" => Strictness::High,
            _ => Strictness::Medium,
        }
    }

    pub fn threshold(&self) -> u32 {
        match self {
            Strictness::Low => 3,
            Strictness::Medium => 2,
            Strictness::High => 1,
        }
    }
}

pub struct MonitorLogic {
    pub consecutive_bad: u32,
    strictness: Strictness,
}

impl MonitorLogic {
    pub fn new(strictness: Strictness) -> Self {
        Self {
            consecutive_bad: 0,
            strictness,
        }
    }

    pub fn set_strictness(&mut self, strictness: Strictness) {
        self.strictness = strictness;
    }

    pub fn process_status(&mut self, status: super::posture::PostureStatus) -> AlertEvent {
        match status {
            super::posture::PostureStatus::Bad => {
                self.consecutive_bad += 1;
                if self.consecutive_bad >= self.strictness.threshold() {
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
            super::posture::PostureStatus::NoPerson => {
                // No person detected - reset counter, no alerts
                println!("No person detected in frame.");
                self.consecutive_bad = 0;
                AlertEvent::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posture::PostureStatus;

    #[test]
    fn test_alert_repeat_behavior_medium() {
        let mut logic = MonitorLogic::new(Strictness::Medium);

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
    }

    #[test]
    fn test_alert_repeat_behavior_high() {
        let mut logic = MonitorLogic::new(Strictness::High);

        // First bad - notify immediately
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_alert_repeat_behavior_low() {
        let mut logic = MonitorLogic::new(Strictness::Low);

        // First bad - warning
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::FirstWarning
        ));

        // Second bad - warning
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::FirstWarning
        ));

        // Third bad - notify
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_no_person_resets_counter() {
        let mut logic = MonitorLogic::new(Strictness::Medium);

        // First bad - warning only
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::FirstWarning
        ));

        // NoPerson should reset counter and not trigger alert
        assert!(matches!(
            logic.process_status(PostureStatus::NoPerson),
            AlertEvent::None
        ));

        // After NoPerson, next bad should be a warning again (counter reset)
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::FirstWarning
        ));
    }
}
