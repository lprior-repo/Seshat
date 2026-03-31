#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
use crate::ui::canvas::domain::{
    parse_event, transition, CanvasError, CanvasEvent, InteractionState, RawEvent,
};

pub struct CanvasTestDsl {
    pub state: Option<InteractionState>,
    pub events: Vec<CanvasEvent>,
    pub last_result: Option<Result<InteractionState, CanvasError>>,
}

impl Default for CanvasTestDsl {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasTestDsl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Some(InteractionState::Idle),
            events: vec![],
            last_result: None,
        }
    }

    #[must_use]
    pub fn given_state(mut self, state: InteractionState) -> Self {
        self.state = Some(state);
        self
    }

    #[must_use]
    pub fn when_raw_event(mut self, raw: RawEvent) -> Self {
        let parsed = parse_event(raw);
        match parsed {
            Ok(event) => {
                let state = self.state.take().unwrap_or(InteractionState::Idle);
                let result = transition(state, event);
                if let Ok(new_state) = &result {
                    self.state = Some(new_state.clone());
                } else {
                    // In case of error, we can't continue transitioning, so state is lost or kept as is?
                    // For the DSL, we just store the error.
                }
                self.last_result = Some(result);
            }
            Err(e) => {
                self.last_result = Some(Err(e));
            }
        }
        self
    }

    #[must_use]
    pub fn when_parsed_event(mut self, event: CanvasEvent) -> Self {
        let state = self.state.take().unwrap_or(InteractionState::Idle);
        let result = transition(state, event);
        if let Ok(new_state) = &result {
            self.state = Some(new_state.clone());
        }
        self.last_result = Some(result);
        self
    }

    #[must_use]
    pub fn then_expect_state(self, expected: InteractionState) -> Self {
        // Properly handle the Result without panicking on unwrap
        let actual = match self.last_result.as_ref() {
            Some(Ok(state)) => state,
            Some(Err(e)) => panic!("then_expect_state called but last result was error: {e:?}"),
            None => panic!("then_expect_state called but no last_result was set"),
        };
        assert_eq!(actual, &expected);
        self
    }

    #[must_use]
    pub fn then_expect_error(self, expected_err: CanvasError) -> Self {
        // Properly handle the Result without panicking on unwrap
        let actual_err = match self.last_result.as_ref() {
            Some(Ok(state)) => {
                panic!("then_expect_error called but last result was Ok state: {state:?}")
            }
            Some(Err(e)) => e,
            None => panic!("then_expect_error called but no last_result was set"),
        };
        assert_eq!(actual_err, &expected_err);
        self
    }
}
