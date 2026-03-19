bead_id: seshat-g1ej
bead_title: "Red Queen: Panics via unwrap() in geometry routing and store bridge"
phase: STATE_1
updated_at: 2026-03-19T03:05:00Z

# State Machine Progress

## STATE 0: ISOLATION & CALIBRATION ✅
- [x] Bead claimed: seshat-g1ej
- [x] JJ workspace at: /home/lewis/src/seshat-g1ej
- [x] STATE.md initialized

## STATE 1: CONTRACT SYNTHESIS (current)
- [ ] Launch rust-contract sub-agent
- [ ] Verify contract.md exists
- [ ] Verify martin-fowler-tests.md exists

## STATE 2: TEST PLAN REVIEW
- [ ] Launch test-reviewer sub-agent

## STATE 3: IMPLEMENTATION
- [ ] Launch functional-rust sub-agent

## STATE 4: MOON GATE
- [ ] Run moon run :quick
- [ ] Run moon run :test
- [ ] Run moon run :ci
- [ ] Run moon run :e2e

## STATE 5: ADVERSARIAL REVIEW (RED QUEEN)
- [ ] Launch red-queen sub-agent

## STATE 5.5: BLACK HAT REVIEW
- [ ] Launch black-hat-reviewer sub-agent

## STATE 5.7: KANI MODEL CHECKING
- [ ] Run cargo kani or provide formal justification

## STATE 6: REPAIR LOOP
- [ ] If needed, launch functional-rust to fix defects

## STATE 7: ARCHITECTURAL DRIFT
- [ ] Launch architectural-drift sub-agent

## STATE 8: LANDING
- [ ] jj rebase -d main@origin
- [ ] jj git push --bookmark main
- [ ] bd close
- [ ] jj workspace forget
