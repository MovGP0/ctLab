//! Aggregates independent test observations so one mismatch does not hide later failures.

use core::fmt::{Debug, Display};
use core::panic::Location;

/// Records failed test observations and reports them together when a test finishes.
#[derive(Debug, Default)]
pub(crate) struct TestFailures {
    messages: Vec<String>,
}

impl TestFailures {
    /// Records an equality mismatch without interrupting the remaining test observations.
    #[track_caller]
    pub(crate) fn eq<Actual, Expected>(
        &mut self,
        actual: Actual,
        expected: Expected,
    ) where
        Actual: Debug + PartialEq<Expected>,
        Expected: Debug,
    {
        self.eq_with_message(actual, expected, "values should be equal");
    }

    /// Records an equality mismatch with domain context while allowing later observations to run.
    #[track_caller]
    pub(crate) fn eq_with_message<Actual, Expected>(
        &mut self,
        actual: Actual,
        expected: Expected,
        description: impl Display,
    ) where
        Actual: Debug + PartialEq<Expected>,
        Expected: Debug,
    {
        if actual != expected {
            self.record(
                description,
                format_args!("  expected: {expected:?}\n    actual: {actual:?}"),
            );
        }
    }

    /// Records an unexpected equality without interrupting the remaining test observations.
    #[track_caller]
    pub(crate) fn ne<Actual, Expected>(
        &mut self,
        actual: Actual,
        unexpected: Expected,
    ) where
        Actual: Debug + PartialEq<Expected>,
        Expected: Debug,
    {
        self.ne_with_message(actual, unexpected, "values should differ");
    }

    /// Records an unexpected equality with domain context while allowing later observations to run.
    #[track_caller]
    pub(crate) fn ne_with_message<Actual, Expected>(
        &mut self,
        actual: Actual,
        unexpected: Expected,
        description: impl Display,
    ) where
        Actual: Debug + PartialEq<Expected>,
        Expected: Debug,
    {
        if actual == unexpected {
            self.record(
                description,
                format_args!("  unexpected: {unexpected:?}\n      actual: {actual:?}"),
            );
        }
    }

    /// Records a false condition without interrupting the remaining test observations.
    #[track_caller]
    pub(crate) fn is_true(&mut self, condition: bool) {
        self.is_true_with_message(condition, "condition should be true");
    }

    /// Records a false condition with domain context while allowing later observations to run.
    #[track_caller]
    pub(crate) fn is_true_with_message(&mut self, condition: bool, description: impl Display) {
        if !condition {
            self.record(description, format_args!("  expected: true\n    actual: false"));
        }
    }

    /// Records a true condition that was expected to be false without interrupting later checks.
    #[track_caller]
    pub(crate) fn is_false(&mut self, condition: bool) {
        self.is_false_with_message(condition, "condition should be false");
    }

    /// Records an unexpected true condition with domain context while allowing later checks to run.
    #[track_caller]
    pub(crate) fn is_false_with_message(&mut self, condition: bool, description: impl Display) {
        if condition {
            self.record(description, format_args!("  expected: false\n    actual: true"));
        }
    }

    /// Records an explicit failure for an observation that cannot be represented as a comparison.
    #[track_caller]
    pub(crate) fn fail(&mut self, description: impl Display) {
        self.record(description, format_args!("  observation failed"));
    }

    /// Fails once with every recorded diagnostic after all independent observations have run.
    #[track_caller]
    pub(crate) fn finish(self) {
        if !self.messages.is_empty() {
            panic!(
                "{} test assertions failed:\n\n{}",
                self.messages.len(),
                self.messages.join("\n\n"),
            );
        }
    }

    #[track_caller]
    fn record(&mut self, description: impl Display, details: impl Display) {
        let location = Location::caller();
        self.messages.push(format!(
            "{}:{}: {description}\n{details}",
            location.file(),
            location.line(),
        ));
    }
}

#[cfg(test)]
#[path = "test_failures_tests.rs"]
mod tests;
