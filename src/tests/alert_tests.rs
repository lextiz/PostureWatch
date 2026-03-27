#[test]
fn notifications_do_not_panic() {
    super::notify_bad_posture();
    super::notify_desk_raise();
}
