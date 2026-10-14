use super::model::AnalyzerFact;
use super::model::ComparisonRow;
use super::model::CorpusFixture;

const OUTCOME_AGREE: &str = "agree";
const OUTCOME_DISAGREE: &str = "disagree";
const OUTCOME_MIRAI_ONLY: &str = "mirai-only";
const OUTCOME_CHARON_ONLY: &str = "charon-only";
const OUTCOME_SOURCE_ONLY: &str = "source-token-only";
const OUTCOME_UNKNOWN: &str = "unknown";

pub fn compare(
    fixture: &CorpusFixture,
    mirai: Option<&AnalyzerFact>,
    charon: Option<&AnalyzerFact>,
    source_token: Option<&AnalyzerFact>,
) -> ComparisonRow {
    let facts = [mirai, charon, source_token];
    let aligned = facts.iter().flatten().all(|fact| fact.source_identity == fixture.source_identity);
    let present = facts.iter().filter(|fact| fact.is_some()).count();
    let outcome = if !aligned {
        OUTCOME_UNKNOWN
    } else {
        match (mirai, charon, source_token) {
            (Some(left), Some(middle), Some(right)) if left.value == middle.value && middle.value == right.value => {
                OUTCOME_AGREE
            }
            (Some(_), Some(_), Some(_)) => OUTCOME_DISAGREE,
            (Some(_), None, None) => OUTCOME_MIRAI_ONLY,
            (None, Some(_), None) => OUTCOME_CHARON_ONLY,
            (None, None, Some(_)) => OUTCOME_SOURCE_ONLY,
            _ if present == 0 => OUTCOME_UNKNOWN,
            _ => OUTCOME_UNKNOWN,
        }
    };
    let mut parent_identities = facts.iter().flatten().map(|fact| fact.parent_identity.clone()).collect::<Vec<_>>();
    parent_identities.sort();
    parent_identities.dedup();
    ComparisonRow {
        fixture_id: fixture.fixture_id.clone(),
        source_identity: fixture.source_identity.clone(),
        property_class: fixture.property_class.clone(),
        outcome: outcome.to_owned(),
        parent_identities,
    }
}
