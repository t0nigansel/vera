use crate::AnswerConfidence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningChoice {
    Recalled,
    Eliminated,
    AppliedRule,
    FromExperience,
    Guessed,
    #[default]
    NotStated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    ConfidentCorrect,
    UnsureCorrect,
    UnsureWrong,
    ConfidentWrong,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttemptDiagnosis {
    pub outcome: AttemptOutcome,
    pub counts_as_mastery: bool,
    pub tutor_recommended: bool,
    pub review_interval_days: i64,
    pub message: String,
}

impl AttemptDiagnosis {
    pub fn evaluate(
        correct: bool,
        confidence: &AnswerConfidence,
        reasoning: ReasoningChoice,
    ) -> Self {
        let outcome = match (correct, confidence) {
            (true, AnswerConfidence::Sure) => AttemptOutcome::ConfidentCorrect,
            (true, _) => AttemptOutcome::UnsureCorrect,
            (false, AnswerConfidence::Sure) => AttemptOutcome::ConfidentWrong,
            (false, _) => AttemptOutcome::UnsureWrong,
        };
        let counts_as_mastery = outcome == AttemptOutcome::ConfidentCorrect
            && !matches!(
                reasoning,
                ReasoningChoice::Eliminated | ReasoningChoice::Guessed
            );
        let tutor_recommended = outcome == AttemptOutcome::ConfidentWrong;
        let review_interval_days = match (outcome, reasoning, confidence) {
            (
                AttemptOutcome::ConfidentCorrect,
                ReasoningChoice::Eliminated | ReasoningChoice::Guessed,
                _,
            ) => 3,
            (AttemptOutcome::ConfidentCorrect, _, _) => 7,
            (AttemptOutcome::UnsureCorrect, _, AnswerConfidence::Guessed) => 1,
            (AttemptOutcome::UnsureCorrect, _, _) => 2,
            (AttemptOutcome::UnsureWrong, _, _) => 1,
            (AttemptOutcome::ConfidentWrong, _, _) => 0,
        };
        let mut message = match (outcome, confidence) {
            (AttemptOutcome::ConfidentCorrect, _) => {
                "Sicher und richtig: Das ist belastbares Wissen und wird später wiederholt."
            }
            (AttemptOutcome::UnsureCorrect, AnswerConfidence::Guessed) => {
                "Richtig geraten: Der Treffer zählt nicht als Beherrschung und kommt bald wieder."
            }
            (AttemptOutcome::UnsureCorrect, _) => {
                "Richtig, aber noch unsicher: Die Antwort zählt noch nicht als sichere Beherrschung."
            }
            (AttemptOutcome::ConfidentWrong, _) => {
                "Sicher und falsch: Hier steckt wahrscheinlich eine konkrete Fehlvorstellung. Kläre sie, bevor du weitergehst."
            }
            (AttemptOutcome::UnsureWrong, AnswerConfidence::Guessed) => {
                "Geraten und falsch: Baue zuerst das zugrunde liegende Lernziel auf."
            }
            (AttemptOutcome::UnsureWrong, _) => {
                "Unsicher und falsch: Die Wissenslücke wird zeitnah erneut eingeplant."
            }
        }
        .to_owned();

        let addition = match (outcome, reasoning) {
            (AttemptOutcome::ConfidentCorrect, ReasoningChoice::Eliminated) => Some(
                "Du hast über Ausschluss entschieden, nicht über die Definition — das trägt in der Prüfung nicht zuverlässig.",
            ),
            (AttemptOutcome::ConfidentCorrect, ReasoningChoice::Guessed) => Some(
                "Du hast nach eigener Angabe geraten, deshalb zählt das nicht als Beherrschung.",
            ),
            (AttemptOutcome::ConfidentWrong, ReasoningChoice::Recalled) => Some(
                "Du hast dich an eine Definition erinnert, die hier nicht zutrifft — genau das ist eine Fehlvorstellung.",
            ),
            (AttemptOutcome::ConfidentWrong, ReasoningChoice::AppliedRule) => Some(
                "Die angewendete Regel passt hier nicht. Prüfe, unter welchen Bedingungen sie gilt.",
            ),
            _ => None,
        };
        if let Some(addition) = addition {
            message.push(' ');
            message.push_str(addition);
        }

        Self {
            outcome,
            counts_as_mastery,
            tutor_recommended,
            review_interval_days,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_all_outcome_cases() {
        for (correct, confidence, expected) in [
            (
                true,
                AnswerConfidence::Sure,
                AttemptOutcome::ConfidentCorrect,
            ),
            (
                true,
                AnswerConfidence::Unsure,
                AttemptOutcome::UnsureCorrect,
            ),
            (
                true,
                AnswerConfidence::Guessed,
                AttemptOutcome::UnsureCorrect,
            ),
            (
                false,
                AnswerConfidence::Sure,
                AttemptOutcome::ConfidentWrong,
            ),
            (false, AnswerConfidence::Unsure, AttemptOutcome::UnsureWrong),
            (
                false,
                AnswerConfidence::Guessed,
                AttemptOutcome::UnsureWrong,
            ),
        ] {
            assert_eq!(
                AttemptDiagnosis::evaluate(correct, &confidence, ReasoningChoice::NotStated)
                    .outcome,
                expected
            );
        }
    }

    #[test]
    fn eliminated_confident_correct_does_not_count_as_mastery() {
        let eliminated =
            AttemptDiagnosis::evaluate(true, &AnswerConfidence::Sure, ReasoningChoice::Eliminated);
        let recalled =
            AttemptDiagnosis::evaluate(true, &AnswerConfidence::Sure, ReasoningChoice::Recalled);

        assert!(!eliminated.counts_as_mastery);
        assert_eq!(eliminated.review_interval_days, 3);
        assert!(recalled.counts_as_mastery);
        assert_eq!(recalled.review_interval_days, 7);
    }

    #[test]
    fn tutor_is_recommended_exactly_for_confident_wrong() {
        for reasoning in [
            ReasoningChoice::Recalled,
            ReasoningChoice::Eliminated,
            ReasoningChoice::AppliedRule,
            ReasoningChoice::FromExperience,
            ReasoningChoice::Guessed,
            ReasoningChoice::NotStated,
        ] {
            for (correct, confidence) in [
                (true, AnswerConfidence::Sure),
                (true, AnswerConfidence::Unsure),
                (true, AnswerConfidence::Guessed),
                (false, AnswerConfidence::Sure),
                (false, AnswerConfidence::Unsure),
                (false, AnswerConfidence::Guessed),
            ] {
                let diagnosis = AttemptDiagnosis::evaluate(correct, &confidence, reasoning);
                assert_eq!(
                    diagnosis.tutor_recommended,
                    diagnosis.outcome == AttemptOutcome::ConfidentWrong
                );
            }
        }
    }

    #[test]
    fn confident_wrong_has_no_review_delay() {
        let diagnosis =
            AttemptDiagnosis::evaluate(false, &AnswerConfidence::Sure, ReasoningChoice::NotStated);

        assert_eq!(diagnosis.review_interval_days, 0);
    }

    #[test]
    fn evaluation_is_deterministic() {
        let first = AttemptDiagnosis::evaluate(
            false,
            &AnswerConfidence::Sure,
            ReasoningChoice::AppliedRule,
        );
        let second = AttemptDiagnosis::evaluate(
            false,
            &AnswerConfidence::Sure,
            ReasoningChoice::AppliedRule,
        );

        assert_eq!(first, second);
    }

    #[test]
    fn additions_appear_only_in_their_exact_combinations() {
        let additions = [
            "Du hast über Ausschluss entschieden, nicht über die Definition — das trägt in der Prüfung nicht zuverlässig.",
            "Du hast nach eigener Angabe geraten, deshalb zählt das nicht als Beherrschung.",
            "Du hast dich an eine Definition erinnert, die hier nicht zutrifft — genau das ist eine Fehlvorstellung.",
            "Die angewendete Regel passt hier nicht. Prüfe, unter welchen Bedingungen sie gilt.",
        ];

        for reasoning in [
            ReasoningChoice::Recalled,
            ReasoningChoice::Eliminated,
            ReasoningChoice::AppliedRule,
            ReasoningChoice::FromExperience,
            ReasoningChoice::Guessed,
            ReasoningChoice::NotStated,
        ] {
            for (correct, confidence) in [
                (true, AnswerConfidence::Sure),
                (true, AnswerConfidence::Unsure),
                (true, AnswerConfidence::Guessed),
                (false, AnswerConfidence::Sure),
                (false, AnswerConfidence::Unsure),
                (false, AnswerConfidence::Guessed),
            ] {
                let diagnosis = AttemptDiagnosis::evaluate(correct, &confidence, reasoning);
                let expected_addition = match (diagnosis.outcome, reasoning) {
                    (AttemptOutcome::ConfidentCorrect, ReasoningChoice::Eliminated) => {
                        Some(additions[0])
                    }
                    (AttemptOutcome::ConfidentCorrect, ReasoningChoice::Guessed) => {
                        Some(additions[1])
                    }
                    (AttemptOutcome::ConfidentWrong, ReasoningChoice::Recalled) => {
                        Some(additions[2])
                    }
                    (AttemptOutcome::ConfidentWrong, ReasoningChoice::AppliedRule) => {
                        Some(additions[3])
                    }
                    _ => None,
                };

                for addition in additions {
                    assert_eq!(
                        diagnosis.message.matches(addition).count(),
                        usize::from(expected_addition == Some(addition))
                    );
                }
            }
        }
    }
}
