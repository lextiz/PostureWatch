pub enum AlertEvent {
    None,
    NotifyBadPosture,
}

pub struct MonitorLogic {
    consecutive_bad: u32,
    threshold: u32,
}

impl MonitorLogic {
    pub fn new(threshold: u32) -> Self {
        Self {
            consecutive_bad: 0,
            threshold: threshold.max(1),
        }
    }

    pub fn set_threshold(&mut self, threshold: u32) {
        self.threshold = threshold.max(1);
    }

    pub fn process_status(&mut self, status: super::posture::PostureStatus) -> AlertEvent {
        match status {
            super::posture::PostureStatus::Bad => {
                self.consecutive_bad += 1;
                if self.consecutive_bad >= self.threshold {
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
    fn test_threshold_2_alerts_on_second_bad() {
        let mut logic = MonitorLogic::new(2);

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
    fn test_threshold_1_alerts_immediately() {
        let mut logic = MonitorLogic::new(1);

        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_threshold_3_alerts_on_third_bad() {
        let mut logic = MonitorLogic::new(3);

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
        let mut logic = MonitorLogic::new(2);

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

    #[test]
    fn test_threshold_zero_becomes_one() {
        let mut logic = MonitorLogic::new(0);
        assert!(matches!(
            logic.process_status(PostureStatus::Bad),
            AlertEvent::NotifyBadPosture
        ));
    }
}
