#[derive(Clone, Copy)]
struct ActorAuthority {
    may_publish: bool,
}

struct PublicationPlanId(u64);

struct AuthorizedPublication(PublicationPlanId);

struct PublicationAuthority(PublicationPlanId);

struct PublicationPlan {
    id: PublicationPlanId,
}

struct PublicationAttempt {
    id: PublicationPlanId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublicationObservation {
    Succeeded,
    Failed,
}

struct PublicationReceipt {
    id: PublicationPlanId,
    observation: PublicationObservation,
}

enum PublicationDecision {
    Authorized {
        plan: PublicationPlan,
        witness: PublicationAuthority,
    },
    Denied,
}

fn decide_publication(authority: ActorAuthority, plan_id: PublicationPlanId) -> PublicationDecision {
    if !authority.may_publish {
        return PublicationDecision::Denied;
    }
    let authorized = AuthorizedPublication(PublicationPlanId(plan_id.0));
    PublicationDecision::Authorized {
        plan: PublicationPlan {
            id: PublicationPlanId(authorized.0.0),
        },
        witness: PublicationAuthority(authorized.0),
    }
}

trait PublicationPort {
    fn publish(
        &mut self,
        plan: PublicationPlan,
        witness: PublicationAuthority,
    ) -> (PublicationAttempt, PublicationObservation);
}

#[derive(Default)]
struct FakePublicationPort {
    calls: usize,
    fail: bool,
}

impl PublicationPort for FakePublicationPort {
    fn publish(
        &mut self,
        plan: PublicationPlan,
        witness: PublicationAuthority,
    ) -> (PublicationAttempt, PublicationObservation) {
        self.calls += 1;
        assert_eq!(plan.id.0, witness.0.0);
        let observation = if self.fail {
            PublicationObservation::Failed
        } else {
            PublicationObservation::Succeeded
        };
        (PublicationAttempt { id: plan.id }, observation)
    }
}

fn run_publication(
    authority: ActorAuthority,
    plan_id: PublicationPlanId,
    port: &mut impl PublicationPort,
) -> Option<PublicationReceipt> {
    let PublicationDecision::Authorized { plan, witness } = decide_publication(authority, plan_id) else {
        return None;
    };
    let (attempt, observation) = port.publish(plan, witness);
    Some(PublicationReceipt {
        id: attempt.id,
        observation,
    })
}

fn main() {
    const SUCCEEDED_PLAN_ID: u64 = 2;
    const FAILED_PLAN_ID: u64 = 3;
    const EXPECTED_CALLS_AFTER_FAILURE: usize = 2;

    let mut port = FakePublicationPort::default();
    let denied = run_publication(ActorAuthority { may_publish: false }, PublicationPlanId(1), &mut port);
    assert!(denied.is_none());
    assert_eq!(port.calls, 0);

    let succeeded =
        run_publication(ActorAuthority { may_publish: true }, PublicationPlanId(SUCCEEDED_PLAN_ID), &mut port)
            .expect("authorized publication has an observation receipt");
    assert_eq!(succeeded.id.0, SUCCEEDED_PLAN_ID);
    assert_eq!(succeeded.observation, PublicationObservation::Succeeded);
    assert_eq!(port.calls, 1);

    port.fail = true;
    let failed = run_publication(ActorAuthority { may_publish: true }, PublicationPlanId(FAILED_PLAN_ID), &mut port)
        .expect("failed execution still has a failure receipt");
    assert_eq!(failed.id.0, FAILED_PLAN_ID);
    assert_eq!(failed.observation, PublicationObservation::Failed);
    assert_eq!(port.calls, EXPECTED_CALLS_AFTER_FAILURE);
}
