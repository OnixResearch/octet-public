use super::model::ConsumerOutcome;
use super::model::OutcomeDecision;
use super::model::StageObservation;
use super::model::StageStatus;

/// Classify typed stage observations without process or filesystem access.
#[must_use]
pub fn classify(observations: &[StageObservation]) -> OutcomeDecision {
    if let Some(observation) = observations.iter().find(|observation| {
        matches!(
            observation.status,
            StageStatus::ProcessFailed
                | StageStatus::PolicyFailed
                | StageStatus::ArtifactFailed
                | StageStatus::TimedOut
                | StageStatus::BudgetExceeded
        )
    }) {
        return decision(ConsumerOutcome::Failed, observation);
    }
    if let Some(observation) = observations.iter().find(|observation| observation.status == StageStatus::Unsupported) {
        let complete = observation.construct.as_ref().is_some_and(|value| !value.trim().is_empty())
            && observation.diagnostic_identity.as_ref().is_some_and(|value| super::is_b3(value))
            && observation.next_action.as_ref().is_some_and(|value| !value.trim().is_empty());
        return if complete {
            decision(ConsumerOutcome::Blocked, observation)
        } else {
            decision(ConsumerOutcome::Failed, observation)
        };
    }
    if !observations.is_empty()
        && observations
            .iter()
            .all(|observation| observation.status == StageStatus::Succeeded && !observation.stage.trim().is_empty())
    {
        OutcomeDecision {
            outcome: ConsumerOutcome::Supported,
            stage: None,
            construct: None,
            diagnostic_identity: None,
            next_action: None,
        }
    } else {
        OutcomeDecision {
            outcome: ConsumerOutcome::Failed,
            stage: None,
            construct: None,
            diagnostic_identity: None,
            next_action: None,
        }
    }
}

fn decision(outcome: ConsumerOutcome, observation: &StageObservation) -> OutcomeDecision {
    OutcomeDecision {
        outcome,
        stage: Some(observation.stage.clone()),
        construct: observation.construct.clone(),
        diagnostic_identity: observation.diagnostic_identity.clone(),
        next_action: observation.next_action.clone(),
    }
}
