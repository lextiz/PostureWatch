use super::{set_current_posture_status, TrayManager, LAST_POSTURE_SCORE};
use crate::config::Config;
use crate::posture::PostureStatus;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[test]
fn setup_tray_is_noop_on_non_windows() {
    let config = Arc::new(TokioMutex::new(Config::default()));
    TrayManager::setup_tray(config);
}

#[test]
fn set_current_posture_status_updates_score_store() {
    set_current_posture_status(&PostureStatus::Score(8));
    assert_eq!(LAST_POSTURE_SCORE.load(Ordering::SeqCst), 8);

    set_current_posture_status(&PostureStatus::NoPerson);
    assert_eq!(LAST_POSTURE_SCORE.load(Ordering::SeqCst), 0);
}
