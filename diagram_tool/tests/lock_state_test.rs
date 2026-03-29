#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

#[test]
fn lock_state_should_exist() {
    use diagram_models::document::LockState;
    assert!(!LockState::Unlocked.is_locked());
    assert!(LockState::Locked.is_locked());
}
