use crate::AttemptDiagnosis;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemKind {
    LearningObjective,
    Term,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewPriority {
    Overdue,
    New,
    Upcoming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    pub item_id: String,
    pub kind: ReviewItemKind,
    pub attempts: i64,
    pub due_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPlanItem {
    pub item_id: String,
    pub kind: ReviewItemKind,
    pub priority: ReviewPriority,
    pub due_at: Option<String>,
    pub reason: String,
}

pub fn next_interval_days(mastery_streak: i64, diagnosis: &AttemptDiagnosis) -> i64 {
    if !diagnosis.counts_as_mastery {
        return diagnosis.review_interval_days;
    }

    let intervals = [7, 14, 30, 60];
    intervals[mastery_streak.clamp(0, intervals.len() as i64 - 1) as usize]
}

pub fn compress_for_exam(days: i64, days_until_exam: Option<i64>) -> i64 {
    if days == 0 {
        return 0;
    }

    match days_until_exam {
        Some(days_until_exam) => days.min((days_until_exam / 3).max(1)),
        None => days,
    }
}

pub fn plan(states: Vec<ReviewState>, now: &str) -> Vec<ReviewPlanItem> {
    let mut items = states
        .into_iter()
        .map(|state| {
            let (priority, reason) = if state.due_at.as_deref().is_some_and(|due_at| due_at <= now)
            {
                (ReviewPriority::Overdue, "Wiederholung fällig")
            } else if state.attempts == 0 {
                (ReviewPriority::New, "Noch nicht geübt")
            } else {
                (ReviewPriority::Upcoming, "Sitzt vorerst")
            };

            ReviewPlanItem {
                item_id: state.item_id,
                kind: state.kind,
                priority,
                due_at: state.due_at,
                reason: reason.into(),
            }
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        let rank = |priority| match priority {
            ReviewPriority::Overdue => 0,
            ReviewPriority::New => 1,
            ReviewPriority::Upcoming => 2,
        };

        rank(left.priority)
            .cmp(&rank(right.priority))
            .then_with(|| match left.priority {
                ReviewPriority::Overdue | ReviewPriority::Upcoming => {
                    left.due_at.cmp(&right.due_at)
                }
                ReviewPriority::New => std::cmp::Ordering::Equal,
            })
            .then_with(|| left.item_id.cmp(&right.item_id))
    });

    items
}

pub fn next_item<'a>(
    plan: &'a [ReviewPlanItem],
    last_item_id: Option<&str>,
) -> Option<&'a ReviewPlanItem> {
    plan.iter()
        .find(|item| Some(item.item_id.as_str()) != last_item_id)
        .or_else(|| plan.first())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnswerConfidence, ReasoningChoice};

    #[test]
    fn non_mastery_keeps_diagnosis_interval_for_high_streak() {
        let diagnosis =
            AttemptDiagnosis::evaluate(true, &AnswerConfidence::Sure, ReasoningChoice::Eliminated);

        assert!(!diagnosis.counts_as_mastery);
        assert_eq!(next_interval_days(99, &diagnosis), 3);
    }

    #[test]
    fn mastery_intervals_follow_the_capped_ladder() {
        let diagnosis =
            AttemptDiagnosis::evaluate(true, &AnswerConfidence::Sure, ReasoningChoice::Recalled);

        for (mastery_streak, expected) in [7, 14, 30, 60, 60].into_iter().enumerate() {
            assert_eq!(
                next_interval_days(mastery_streak as i64, &diagnosis),
                expected
            );
        }
    }

    #[test]
    fn exam_compression_respects_boundaries() {
        assert_eq!(compress_for_exam(30, None), 30);
        assert_eq!(compress_for_exam(30, Some(30)), 10);
        assert_eq!(compress_for_exam(0, Some(30)), 0);
        assert_eq!(compress_for_exam(30, Some(-1)), 1);
    }

    #[test]
    fn plan_orders_priorities_due_dates_and_item_ids() {
        let states = vec![
            state("upcoming-z", 1, Some("2026-09-03T00:00:00+00:00")),
            state("new-z", 0, None),
            state("overdue-z", 1, Some("2026-08-01T00:00:00+00:00")),
            state("upcoming-a", 1, Some("2026-09-02T00:00:00+00:00")),
            state("new-a", 0, None),
            state("overdue-a", 1, Some("2026-07-31T00:00:00+00:00")),
        ];

        let plan = plan(states, "2026-08-02T00:00:00+00:00");

        assert_eq!(
            plan.iter()
                .map(|item| item.item_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "overdue-a",
                "overdue-z",
                "new-a",
                "new-z",
                "upcoming-a",
                "upcoming-z"
            ]
        );
        assert_eq!(plan[0].priority, ReviewPriority::Overdue);
        assert_eq!(plan[2].priority, ReviewPriority::New);
        assert_eq!(plan[4].priority, ReviewPriority::Upcoming);
    }

    #[test]
    fn next_item_avoids_last_item_unless_it_is_the_only_one() {
        let plan = plan(
            vec![state("item-a", 0, None), state("item-b", 0, None)],
            "2026-08-02T00:00:00+00:00",
        );

        assert_eq!(next_item(&plan, Some("item-a")).unwrap().item_id, "item-b");
        assert_eq!(
            next_item(&plan[..1], Some("item-a")).unwrap().item_id,
            "item-a"
        );
        assert!(next_item(&[], None).is_none());
    }

    fn state(item_id: &str, attempts: i64, due_at: Option<&str>) -> ReviewState {
        ReviewState {
            item_id: item_id.into(),
            kind: ReviewItemKind::LearningObjective,
            attempts,
            due_at: due_at.map(str::to_owned),
        }
    }
}
