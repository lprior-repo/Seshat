use crate::ui::canvas::domain::{
    parse_event, transition, CanvasError, CanvasEvent, InteractionState, RawEvent,
};

pub struct CanvasTestDsl {
    pub state: Option<InteractionState>,
    pub events: Vec<CanvasEvent>,
    pub last_result: Option<Result<InteractionState, CanvasError>>,
}

impl CanvasTestDsl {
    pub fn new() -> Self {
        Self {
            state: Some(InteractionState::Idle),
            events: vec![],
            last_result: None,
        }
    }

    pub fn given_state(mut self, state: InteractionState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn when_raw_event(mut self, raw: RawEvent) -> Self {
        let parsed = parse_event(raw);
        match parsed {
            Ok(event) => {
                let state = self.state.take().unwrap_or(InteractionState::Idle);
                let result = transition(state, event);
                match &result {
                    Ok(new_state) => {
                        self.state = Some(new_state.clone());
                    }
                    Err(_) => {
                        // In case of error, we can't continue transitioning, so state is lost or kept as is?
                        // For the DSL, we just store the error.
                    }
                }
                self.last_result = Some(result);
            }
            Err(e) => {
                self.last_result = Some(Err(e));
            }
        }
        self
    }

    pub fn when_parsed_event(mut self, event: CanvasEvent) -> Self {
        let state = self.state.take().unwrap_or(InteractionState::Idle);
        let result = transition(state, event);
        match &result {
            Ok(new_state) => {
                self.state = Some(new_state.clone());
            }
            Err(_) => {}
        }
        self.last_result = Some(result);
        self
    }

    pub fn then_expect_state(self, expected: InteractionState) -> Self {
        let actual = self.last_result.as_ref().unwrap().as_ref().unwrap();
        assert_eq!(actual, &expected);
        self
    }

    pub fn then_expect_error(self, expected_err: CanvasError) -> Self {
        let actual_err = self.last_result.as_ref().unwrap().as_ref().unwrap_err();
        assert_eq!(actual_err, &expected_err);
        self
    }
}
