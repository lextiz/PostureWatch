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
#[path = "tests/posture_monitor_tests.rs"]
mod tests;
