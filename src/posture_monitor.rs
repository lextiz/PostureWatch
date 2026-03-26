pub enum AlertEvent {
    None,
    NotifyBadPosture,
}

pub struct MonitorLogic {
    consecutive_bad: u32,
    posture_threshold: u32,
    alert_threshold: u32,
}

impl MonitorLogic {
    pub fn new(posture_threshold: u32, alert_threshold: u32) -> Self {
        Self {
            consecutive_bad: 0,
            posture_threshold: posture_threshold.clamp(1, 10),
            alert_threshold: alert_threshold.max(1),
        }
    }

    pub fn set_thresholds(&mut self, posture_threshold: u32, alert_threshold: u32) {
        self.posture_threshold = posture_threshold.clamp(1, 10);
        self.alert_threshold = alert_threshold.max(1);
    }

    pub fn process_status(&mut self, status: super::posture::PostureStatus) -> AlertEvent {
        match status {
            super::posture::PostureStatus::Score(score) => {
                if score < self.posture_threshold {
                    self.consecutive_bad += 1;
                    if self.consecutive_bad >= self.alert_threshold {
                        self.consecutive_bad = 0; // Reset after alerting
                        AlertEvent::NotifyBadPosture
                    } else {
                        AlertEvent::None
                    }
                } else {
                    self.consecutive_bad = 0;
                    AlertEvent::None
                }
            }
            super::posture::PostureStatus::NoPerson => {
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
    fn test_score_below_threshold_triggers_alert() {
        let mut logic = MonitorLogic::new(5, 2);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(7)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(4)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(3)),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_score_at_threshold_is_good() {
        let mut logic = MonitorLogic::new(5, 1);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(5)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(4)),
            AlertEvent::NotifyBadPosture
        ));
    }

    #[test]
    fn test_good_score_resets_counter() {
        let mut logic = MonitorLogic::new(5, 2);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(3)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(6)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(3)),
            AlertEvent::None
        ));
    }

    #[test]
    fn test_no_person_resets_counter() {
        let mut logic = MonitorLogic::new(5, 2);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(3)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::NoPerson),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(3)),
            AlertEvent::None
        ));
    }

    #[test]
    fn test_thresholds_are_clamped() {
        let mut logic = MonitorLogic::new(0, 0);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(1)),
            AlertEvent::NotifyBadPosture
        ));

        logic.set_thresholds(42, 0);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(9)),
            AlertEvent::NotifyBadPosture
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(10)),
            AlertEvent::None
        ));
    }

    #[test]
    fn test_counter_resets_after_alert() {
        let mut logic = MonitorLogic::new(5, 2);

        assert!(matches!(
            logic.process_status(PostureStatus::Score(4)),
            AlertEvent::None
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(4)),
            AlertEvent::NotifyBadPosture
        ));
        assert!(matches!(
            logic.process_status(PostureStatus::Score(4)),
            AlertEvent::None
        ));
    }
}
