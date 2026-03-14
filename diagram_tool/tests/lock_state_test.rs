#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

#[test]
fn lock_state_should_exist() {
    use diagram_tool::models::document::LockState;
    assert!(!LockState::Unlocked.is_locked());
    assert!(LockState::Locked.is_locked());
}
