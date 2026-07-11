use super::TestFailures;

/// Verifies that independent mismatches remain available until the test explicitly finishes.
#[test]
fn records_multiple_failures_before_finishing() {
    let mut assert = TestFailures::default();

    let mut observed = TestFailures::default();
    observed.eq(1, 2);
    observed.is_true(false);

    assert.eq(observed.messages.len(), 2);
    assert.is_true(
        observed
            .messages
            .first()
            .is_some_and(|message| message.contains("values should be equal")),
    );
    assert.is_true(
        observed
            .messages
            .get(1)
            .is_some_and(|message| message.contains("condition should be true")),
    );
    assert.finish();
}

/// Verifies that finishing reports every recorded mismatch in one final panic.
#[test]
fn finish_reports_every_recorded_failure() {
    let mut assert = TestFailures::default();

    let result = std::panic::catch_unwind(|| {
        let mut observed = TestFailures::default();
        observed.eq("first actual", "first expected");
        observed.eq("second actual", "second expected");
        observed.finish();
    });
    let message = result
        .err()
        .and_then(|payload| payload.downcast::<String>().ok())
        .map(|message| *message);

    assert.is_true(message.as_ref().is_some_and(|text| text.contains("2 test assertions failed")));
    assert.is_true(message.as_ref().is_some_and(|text| text.contains("first expected")));
    assert.is_true(message.as_ref().is_some_and(|text| text.contains("second expected")));
    assert.finish();
}
