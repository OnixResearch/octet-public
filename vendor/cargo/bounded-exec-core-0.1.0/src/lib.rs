//! Deterministic validation and outcome classification for bounded execution.
//!
//! This crate is `no_std` plus `alloc`. It does not spawn processes, inspect
//! clocks, read environments, capture output, or assign evidence authority.

// Publication markers link the core source to retained distribution evidence.
// They do not promote distribution evidence into core behavioral proof.
// r[verify bounded_exec.radicle_publication.identity]
// r[verify bounded_exec.radicle_publication.identity.scenario.accepted]
// r[verify bounded_exec.radicle_publication.identity.scenario.endpoint-only]
// r[verify bounded_exec.radicle_publication.availability]
// r[verify bounded_exec.radicle_publication.availability.scenario.seed-outage]
// r[verify bounded_exec.radicle_publication.availability.scenario.no-fallback]
// r[verify bounded_exec.radicle_publication.history]
// r[verify bounded_exec.radicle_publication.history.scenario.exact-revision]
// r[verify bounded_exec.radicle_publication.history.scenario.rewrite-rejected]
// r[verify bounded_exec.radicle_publication.consumer]
// r[verify bounded_exec.radicle_publication.consumer.scenario.lattice]
// r[verify bounded_exec.radicle_publication.consumer.scenario.mismatch]
// r[verify bounded_exec.radicle_publication.evidence]
// r[verify bounded_exec.radicle_publication.evidence.scenario.valid]
// r[verify bounded_exec.radicle_publication.evidence.scenario.invalid]

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;

mod watermark;

pub use watermark::*;

const ADJACENT_PAIR_ITEMS: usize = 2;

/// Resource and timing limits for one child-process execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionLimits {
    /// Maximum execution time in milliseconds.
    pub timeout_ms: u64,
    /// Maximum stdin bytes accepted from the caller.
    pub stdin_max_bytes: usize,
    /// Maximum stdout bytes retained in memory.
    pub stdout_max_bytes: usize,
    /// Maximum stderr bytes retained in memory.
    pub stderr_max_bytes: usize,
    /// Interval between child-status polls in milliseconds.
    pub poll_interval_ms: u64,
    /// Maximum time to wait for I/O worker teardown in milliseconds.
    pub teardown_timeout_ms: u64,
}

impl ExecutionLimits {
    /// Validate that every bound is positive and that polling cannot exceed the
    /// execution deadline.
    ///
    /// # Errors
    ///
    /// Returns the first invalid limit in declaration order.
    pub const fn validate(self) -> Result<(), LimitError> {
        if self.timeout_ms == 0 {
            return Err(LimitError::TimeoutZero);
        }
        if self.stdin_max_bytes == 0 {
            return Err(LimitError::StdinZero);
        }
        if self.stdout_max_bytes == 0 {
            return Err(LimitError::StdoutZero);
        }
        if self.stderr_max_bytes == 0 {
            return Err(LimitError::StderrZero);
        }
        if self.poll_interval_ms == 0 {
            return Err(LimitError::PollIntervalZero);
        }
        if self.poll_interval_ms > self.timeout_ms {
            return Err(LimitError::PollIntervalExceedsTimeout);
        }
        if self.teardown_timeout_ms == 0 {
            return Err(LimitError::TeardownTimeoutZero);
        }
        Ok(())
    }
}

/// Invalid execution-limit reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitError {
    /// The execution timeout is zero.
    TimeoutZero,
    /// The stdin acceptance bound is zero.
    StdinZero,
    /// The stdout retention bound is zero.
    StdoutZero,
    /// The stderr retention bound is zero.
    StderrZero,
    /// The status-poll interval is zero.
    PollIntervalZero,
    /// The poll interval is longer than the execution timeout.
    PollIntervalExceedsTimeout,
    /// The teardown timeout is zero.
    TeardownTimeoutZero,
}

/// Final bounded capture facts for one output stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedOutput {
    /// Retained prefix, never longer than the configured stream limit.
    pub bytes: Vec<u8>,
    /// Total bytes observed before the stream closed.
    pub observed_bytes: usize,
    /// Whether bytes beyond the retained prefix were discarded.
    pub truncated: bool,
}

/// Pure bounded-retention state for one output stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureAccumulator {
    retained: Vec<u8>,
    observed_bytes: usize,
    limit_bytes: usize,
    prepared_capacity: usize,
}

impl CaptureAccumulator {
    /// Reserve the complete explicit retention limit before observation.
    ///
    /// # Errors
    ///
    /// Returns [`CapturePrepareError`] when the limit is zero or the allocator
    /// cannot reserve the requested capacity.
    ///
    /// # Panics
    ///
    /// Panics only if a successful vector reservation reports less capacity
    /// than requested.
    // r[impl bounded_exec.capture_capacity.preparation]
    pub fn prepare(limit_bytes: usize) -> Result<Self, CapturePrepareError> {
        if limit_bytes == 0 {
            return Err(CapturePrepareError::new(limit_bytes));
        }

        let mut retained = Vec::new();
        retained
            .try_reserve_exact(limit_bytes)
            .map_err(|_error| CapturePrepareError::new(limit_bytes))?;
        let prepared_capacity = retained.capacity();
        assert!(
            prepared_capacity >= limit_bytes,
            "successful reservation must cover the declared limit"
        );

        Ok(Self {
            retained,
            observed_bytes: 0,
            limit_bytes,
            prepared_capacity,
        })
    }

    /// Return the declared retention limit.
    #[must_use]
    pub const fn limit_bytes(&self) -> usize {
        self.limit_bytes
    }

    /// Return the capacity fixed during preparation.
    #[must_use]
    pub const fn prepared_capacity(&self) -> usize {
        self.prepared_capacity
    }

    /// Observe one byte chunk while retaining at most the configured prefix.
    ///
    /// # Errors
    ///
    /// Returns an error if the total observed-byte counter overflows.
    ///
    /// # Panics
    ///
    /// Panics if internal prepared-capacity invariants were violated.
    // r[impl bounded_exec.capture_capacity.steady_state]
    pub fn observe(&mut self, bytes: &[u8]) -> Result<(), CaptureError> {
        self.observed_bytes = self
            .observed_bytes
            .checked_add(bytes.len())
            .ok_or(CaptureError::ObservedBytesOverflow)?;
        let remaining_bytes = self.limit_bytes.saturating_sub(self.retained.len());
        let retained_now = bytes.len().min(remaining_bytes);
        assert!(
            self.retained.len() + retained_now <= self.prepared_capacity,
            "retained bytes must fit prepared capacity"
        );
        self.retained.extend_from_slice(&bytes[..retained_now]);
        assert_eq!(
            self.retained.capacity(),
            self.prepared_capacity,
            "capture observation must not grow capacity"
        );
        Ok(())
    }

    /// Finish the capture and derive its truncation fact.
    ///
    /// # Panics
    ///
    /// Panics if internal retention or prepared-capacity invariants were
    /// violated.
    #[must_use]
    pub fn finish(self) -> CapturedOutput {
        let truncated = self.observed_bytes > self.retained.len();
        assert!(self.retained.len() <= self.limit_bytes);
        assert_eq!(self.retained.capacity(), self.prepared_capacity);
        assert!(!truncated || self.retained.len() == self.limit_bytes);
        CapturedOutput {
            bytes: self.retained,
            observed_bytes: self.observed_bytes,
            truncated,
        }
    }
}

/// Failure to reserve one explicit output-retention limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapturePrepareError {
    requested_capacity: usize,
}

impl CapturePrepareError {
    const fn new(requested_capacity: usize) -> Self {
        Self { requested_capacity }
    }

    /// Return the capacity that preparation could not reserve.
    #[must_use]
    pub const fn requested_capacity(self) -> usize {
        self.requested_capacity
    }
}

/// Pure capture-state failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureError {
    /// The total observed-byte count cannot be represented by `usize`.
    ObservedBytesOverflow,
}

/// Why execution stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completion {
    /// The child exited without runner intervention.
    Exited,
    /// The execution deadline expired.
    TimedOut,
    /// An explicit cancellation request stopped execution.
    Cancelled,
}

/// Output streams whose retention bounds were exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverflowedStreams {
    /// Only stdout exceeded its bound.
    Stdout,
    /// Only stderr exceeded its bound.
    Stderr,
    /// Both streams exceeded their bounds.
    Both,
}

/// Product-neutral execution disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    /// The child exited with an accepted code and satisfied output policy.
    Succeeded,
    /// The child exited with an unaccepted code or without an exit code.
    ExitFailed,
    /// The execution deadline expired.
    TimedOut,
    /// Execution was explicitly cancelled.
    Cancelled,
    /// Output was captured safely, but policy rejects the observed truncation.
    OutputLimitExceeded(OverflowedStreams),
}

/// Explicit facts needed to classify one execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutcomeFacts {
    /// Why execution stopped.
    pub completion: Completion,
    /// Portable exit code, if the platform supplied one.
    pub exit_code: Option<i32>,
    /// Whether stdout exceeded its retention bound.
    pub stdout_truncated: bool,
    /// Whether stderr exceeded its retention bound.
    pub stderr_truncated: bool,
}

/// Consumer-owned policy for interpreting captured execution facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomePolicy {
    accepted_exit_codes: Vec<i32>,
    reject_stdout_truncation: bool,
    reject_stderr_truncation: bool,
}

impl OutcomePolicy {
    /// Construct and validate an outcome policy.
    ///
    /// # Errors
    ///
    /// Returns an error when the accepted-code set is empty or contains a
    /// duplicate code.
    pub fn new(
        mut accepted_exit_codes: Vec<i32>,
        reject_stdout_truncation: bool,
        reject_stderr_truncation: bool,
    ) -> Result<Self, PolicyError> {
        if accepted_exit_codes.is_empty() {
            return Err(PolicyError::AcceptedExitCodesEmpty);
        }
        accepted_exit_codes.sort_unstable();
        if accepted_exit_codes
            .windows(ADJACENT_PAIR_ITEMS)
            .any(|codes| codes[0] == codes[1])
        {
            return Err(PolicyError::AcceptedExitCodeDuplicate);
        }
        Ok(Self {
            accepted_exit_codes,
            reject_stdout_truncation,
            reject_stderr_truncation,
        })
    }

    /// Return the accepted portable exit codes.
    #[must_use]
    pub fn accepted_exit_codes(&self) -> &[i32] {
        &self.accepted_exit_codes
    }

    /// Return whether stdout truncation is rejected.
    #[must_use]
    pub const fn rejects_stdout_truncation(&self) -> bool {
        self.reject_stdout_truncation
    }

    /// Return whether stderr truncation is rejected.
    #[must_use]
    pub const fn rejects_stderr_truncation(&self) -> bool {
        self.reject_stderr_truncation
    }
}

/// Invalid outcome-policy reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    /// At least one accepted exit code is required.
    AcceptedExitCodesEmpty,
    /// Accepted exit codes must not contain duplicates.
    AcceptedExitCodeDuplicate,
}

/// Classify explicit execution facts under consumer-owned policy.
#[must_use]
pub fn classify(facts: OutcomeFacts, policy: &OutcomePolicy) -> Disposition {
    match facts.completion {
        Completion::TimedOut => return Disposition::TimedOut,
        Completion::Cancelled => return Disposition::Cancelled,
        Completion::Exited => {}
    }

    let rejected_stdout = facts.stdout_truncated && policy.rejects_stdout_truncation();
    let rejected_stderr = facts.stderr_truncated && policy.rejects_stderr_truncation();
    let overflow = match (rejected_stdout, rejected_stderr) {
        (true, true) => Some(OverflowedStreams::Both),
        (true, false) => Some(OverflowedStreams::Stdout),
        (false, true) => Some(OverflowedStreams::Stderr),
        (false, false) => None,
    };
    if let Some(streams) = overflow {
        return Disposition::OutputLimitExceeded(streams);
    }

    if facts
        .exit_code
        .is_some_and(|code| policy.accepted_exit_codes().contains(&code))
    {
        Disposition::Succeeded
    } else {
        Disposition::ExitFailed
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const SUCCESS_EXIT_CODE: i32 = 0;
    const ALTERNATE_EXIT_CODE: i32 = 2;
    const TEST_TIMEOUT_MS: u64 = 1_000;
    const TEST_STREAM_MAX_BYTES: usize = 4_096;
    const TEST_POLL_INTERVAL_MS: u64 = 10;
    const TEST_TEARDOWN_TIMEOUT_MS: u64 = 500;

    const VALID_LIMITS: ExecutionLimits = ExecutionLimits {
        timeout_ms: TEST_TIMEOUT_MS,
        stdin_max_bytes: TEST_STREAM_MAX_BYTES,
        stdout_max_bytes: TEST_STREAM_MAX_BYTES,
        stderr_max_bytes: TEST_STREAM_MAX_BYTES,
        poll_interval_ms: TEST_POLL_INTERVAL_MS,
        teardown_timeout_ms: TEST_TEARDOWN_TIMEOUT_MS,
    };

    fn strict_policy() -> OutcomePolicy {
        OutcomePolicy::new(vec![SUCCESS_EXIT_CODE], true, true)
            .expect("strict fixture policy is valid")
    }

    #[test]
    fn valid_limits_and_accepted_exit_succeed() {
        assert_eq!(VALID_LIMITS.validate(), Ok(()));
        let facts = OutcomeFacts {
            completion: Completion::Exited,
            exit_code: Some(SUCCESS_EXIT_CODE),
            stdout_truncated: false,
            stderr_truncated: false,
        };

        assert_eq!(classify(facts, &strict_policy()), Disposition::Succeeded);
    }

    #[test]
    fn invalid_limits_and_policy_fail_closed() {
        let mut zero_timeout = VALID_LIMITS;
        zero_timeout.timeout_ms = 0;
        let mut long_poll = VALID_LIMITS;
        long_poll.poll_interval_ms = TEST_TIMEOUT_MS + 1;

        assert_eq!(zero_timeout.validate(), Err(LimitError::TimeoutZero));
        assert_eq!(
            long_poll.validate(),
            Err(LimitError::PollIntervalExceedsTimeout)
        );
        assert_eq!(
            OutcomePolicy::new(Vec::new(), true, true),
            Err(PolicyError::AcceptedExitCodesEmpty)
        );
        assert_eq!(
            OutcomePolicy::new(vec![SUCCESS_EXIT_CODE, SUCCESS_EXIT_CODE], true, true),
            Err(PolicyError::AcceptedExitCodeDuplicate)
        );
    }

    // r[verify bounded_exec.capture_capacity.steady_state]
    // r[verify bounded_exec.capture_capacity.verification]
    #[test]
    fn capture_retains_a_prefix_without_growing_and_reports_overflow() {
        const CAPTURE_LIMIT_BYTES: usize = 4;
        const FIRST_CHUNK: &[u8] = b"abc";
        const SECOND_CHUNK: &[u8] = b"def";
        let mut capture = CaptureAccumulator::prepare(CAPTURE_LIMIT_BYTES)
            .expect("fixture capacity must prepare");
        let prepared_capacity = capture.prepared_capacity();
        capture
            .observe(FIRST_CHUNK)
            .expect("first capture chunk fits the byte counter");
        assert_eq!(capture.prepared_capacity(), prepared_capacity);
        capture
            .observe(SECOND_CHUNK)
            .expect("second capture chunk fits the byte counter");
        assert_eq!(capture.prepared_capacity(), prepared_capacity);
        let finished = capture.finish();
        let mut overflow = CaptureAccumulator {
            retained: Vec::with_capacity(CAPTURE_LIMIT_BYTES),
            observed_bytes: usize::MAX,
            limit_bytes: CAPTURE_LIMIT_BYTES,
            prepared_capacity: CAPTURE_LIMIT_BYTES,
        };

        assert_eq!(finished.bytes, b"abcd");
        assert_eq!(
            finished.observed_bytes,
            FIRST_CHUNK.len() + SECOND_CHUNK.len()
        );
        assert!(finished.truncated);
        assert_eq!(
            overflow.observe(&[0]),
            Err(CaptureError::ObservedBytesOverflow)
        );
    }

    // r[verify bounded_exec.capture_capacity.preparation]
    // r[verify bounded_exec.capture_capacity.boundary]
    // r[verify bounded_exec.capture_capacity.verification]
    #[test]
    fn capture_preparation_is_explicit_and_fails_without_a_fallback() {
        const CAPTURE_LIMIT_BYTES: usize = 4;
        let prepared = CaptureAccumulator::prepare(CAPTURE_LIMIT_BYTES)
            .expect("fixture capacity must prepare");
        let zero = CaptureAccumulator::prepare(0).expect_err("zero limit must fail");
        let impossible = CaptureAccumulator::prepare(usize::MAX)
            .expect_err("unrepresentable capacity must fail");

        assert_eq!(prepared.limit_bytes(), CAPTURE_LIMIT_BYTES);
        assert!(prepared.prepared_capacity() >= CAPTURE_LIMIT_BYTES);
        assert_eq!(zero.requested_capacity(), 0);
        assert_eq!(impossible.requested_capacity(), usize::MAX);
    }

    #[test]
    fn exact_limit_output_is_retained_without_growth_or_truncation() {
        const EXACT_OUTPUT: &[u8] = b"exact";
        let mut capture = CaptureAccumulator::prepare(EXACT_OUTPUT.len())
            .expect("exact output capacity must prepare");
        let prepared_capacity = capture.prepared_capacity();

        capture
            .observe(EXACT_OUTPUT)
            .expect("exact output must fit the byte counter");
        assert_eq!(capture.prepared_capacity(), prepared_capacity);
        let finished = capture.finish();

        assert_eq!(finished.bytes, EXACT_OUTPUT);
        assert_eq!(finished.observed_bytes, EXACT_OUTPUT.len());
        assert!(!finished.truncated);
    }

    #[test]
    fn timeout_and_cancellation_take_precedence_over_output_policy() {
        let timed_out = OutcomeFacts {
            completion: Completion::TimedOut,
            exit_code: None,
            stdout_truncated: true,
            stderr_truncated: true,
        };
        let cancelled = OutcomeFacts {
            completion: Completion::Cancelled,
            ..timed_out
        };

        assert_eq!(classify(timed_out, &strict_policy()), Disposition::TimedOut);
        assert_eq!(
            classify(cancelled, &strict_policy()),
            Disposition::Cancelled
        );
    }

    #[test]
    fn strict_and_truncating_consumers_classify_the_same_capture_differently() {
        let facts = OutcomeFacts {
            completion: Completion::Exited,
            exit_code: Some(SUCCESS_EXIT_CODE),
            stdout_truncated: true,
            stderr_truncated: false,
        };
        let truncating_policy = OutcomePolicy::new(vec![SUCCESS_EXIT_CODE], false, false)
            .expect("truncating fixture policy is valid");

        assert_eq!(
            classify(facts, &strict_policy()),
            Disposition::OutputLimitExceeded(OverflowedStreams::Stdout)
        );
        assert_eq!(classify(facts, &truncating_policy), Disposition::Succeeded);
    }

    #[test]
    fn unaccepted_or_missing_exit_code_fails() {
        let alternate = OutcomeFacts {
            completion: Completion::Exited,
            exit_code: Some(ALTERNATE_EXIT_CODE),
            stdout_truncated: false,
            stderr_truncated: false,
        };
        let signalled = OutcomeFacts {
            exit_code: None,
            ..alternate
        };

        assert_eq!(
            classify(alternate, &strict_policy()),
            Disposition::ExitFailed
        );
        assert_eq!(
            classify(signalled, &strict_policy()),
            Disposition::ExitFailed
        );
    }
}
