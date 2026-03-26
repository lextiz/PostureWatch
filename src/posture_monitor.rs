pub enum AlertEvent {
    None,
    NotifyBadPosture,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Strictness {
    Low,
    Medium,
    High,
}

impl Strictness {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "high" => Self::High,
            _ => Self::Medium,
        }
    }

    fn threshold(self) -> u32 {
        match self {
            Self::Low => 3,
            Self::Medium => 2,
            Self::High => 1,
        }
    }
}

pub struct MonitorLogic {
    consecutive_bad: u32,
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
                    AlertEvent::None
                }
            }
            super::posture::PostureStatus::Good | super::posture::PostureStatus::NoPerson => {
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
    fn test_medium_strictness_alerts_on_second_bad() {
        let mut logic = MonitorLogic::new(Strictness::Medium);

        assert!(matches!(
            logic.process_status(PostureStatus::Good),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_high_strictness_alerts_immediately() {
        let mut logic = MonitorLogic::new(Strictness::High);

        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_low_strictness_alerts_on_third_bad() {
        let mut logic = MonitorLogic::new(Strictness::Low);

        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_no_person_resets_counter() {
        let mut logic = MonitorLogic::new(Strictness::Medium);

        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::NoPerson),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::None
        ));
    }
}
