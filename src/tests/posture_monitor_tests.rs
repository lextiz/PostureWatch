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
        AlertEvent::None
    ));
    assert!(matches!(
        logic.process_status(PostureStatus::Score(0)),
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
