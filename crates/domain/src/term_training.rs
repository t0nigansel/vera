use crate::{AnswerConfidence, AttemptOutcome, SourceReference};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TermDirection {
    TermToDefinition,
    DefinitionToTerm,
    ScenarioToTerm,
    TermToTopic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistractorSource {
    Cluster,
    SeeAlso,
    Chapter,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermCandidate {
    pub id: String,
    pub term: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermTopic {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermCard {
    pub term_id: String,
    pub term: String,
    pub definition: String,
    pub topic: Option<TermTopic>,
    pub cluster_title: Option<String>,
    pub distinction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermExerciseInput {
    pub card: TermCard,
    pub cluster_mates: Vec<TermCandidate>,
    pub see_also: Vec<TermCandidate>,
    pub chapter_mates: Vec<TermCandidate>,
    pub topics: Vec<TermTopic>,
    pub attempt_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermExerciseOption {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermExercise {
    pub term_id: String,
    pub direction: TermDirection,
    pub instruction: String,
    pub prompt: String,
    pub options: Vec<TermExerciseOption>,
    pub distractor_source: DistractorSource,
    pub cluster_title: Option<String>,
    #[serde(skip_serializing)]
    pub correct_option_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermAttemptSubmission {
    pub direction: TermDirection,
    pub selected_option_id: String,
    pub confidence: AnswerConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermAttemptResult {
    pub attempt_id: String,
    pub correct: bool,
    pub correct_option_id: String,
    pub term: String,
    pub definition: String,
    pub outcome: AttemptOutcome,
    pub counts_as_mastery: bool,
    pub confidence: AnswerConfidence,
    pub diagnosis: String,
    pub distinction: Option<String>,
    pub cluster_title: Option<String>,
    pub review_due_at: String,
    pub tutor_recommended: bool,
    pub source: SourceReference,
}

pub fn available_directions(input: &TermExerciseInput) -> Vec<TermDirection> {
    let has_candidate = !term_distractors(input).0.is_empty();
    let mut directions = Vec::with_capacity(4);

    if has_candidate {
        directions.push(TermDirection::TermToDefinition);
        directions.push(TermDirection::DefinitionToTerm);
    }
    if has_candidate
        && input
            .card
            .distinction
            .as_deref()
            .is_some_and(|distinction| !distinction.is_empty())
    {
        directions.push(TermDirection::ScenarioToTerm);
    }
    if input.card.topic.is_some() && input.topics.len() >= 2 {
        directions.push(TermDirection::TermToTopic);
    }

    directions
}

pub fn build_exercise(input: &TermExerciseInput, direction: TermDirection) -> Option<TermExercise> {
    if !available_directions(input).contains(&direction) {
        return None;
    }

    let (instruction, prompt, mut options, distractor_source, correct_option_id) = match direction {
        TermDirection::TermToDefinition => {
            let (distractors, source) = term_distractors(input);
            let mut options = vec![TermExerciseOption {
                id: input.card.term_id.clone(),
                text: input.card.definition.clone(),
            }];
            options.extend(distractors.into_iter().map(|candidate| TermExerciseOption {
                id: candidate.id,
                text: candidate.definition,
            }));
            (
                "Welche Definition gehört zu diesem Begriff?",
                input.card.term.clone(),
                options,
                source,
                input.card.term_id.clone(),
            )
        }
        TermDirection::DefinitionToTerm => {
            let (distractors, source) = term_distractors(input);
            let mut options = vec![TermExerciseOption {
                id: input.card.term_id.clone(),
                text: input.card.term.clone(),
            }];
            options.extend(distractors.into_iter().map(|candidate| TermExerciseOption {
                id: candidate.id,
                text: candidate.term,
            }));
            (
                "Welcher Begriff wird hier definiert?",
                input.card.definition.clone(),
                options,
                source,
                input.card.term_id.clone(),
            )
        }
        TermDirection::ScenarioToTerm => {
            let (distractors, source) = term_distractors(input);
            let mut options = vec![TermExerciseOption {
                id: input.card.term_id.clone(),
                text: input.card.term.clone(),
            }];
            options.extend(distractors.into_iter().map(|candidate| TermExerciseOption {
                id: candidate.id,
                text: candidate.term,
            }));
            (
                "Welcher Begriff ist hier gemeint?",
                input.card.distinction.clone()?,
                options,
                source,
                input.card.term_id.clone(),
            )
        }
        TermDirection::TermToTopic => {
            let topic = input.card.topic.as_ref()?;
            let mut topics = input.topics.iter().collect::<Vec<_>>();
            topics.sort_by(|left, right| left.id.cmp(&right.id));
            let mut used = HashSet::from([topic.id.as_str()]);
            let mut options = vec![TermExerciseOption {
                id: topic.id.clone(),
                text: topic.title.clone(),
            }];
            for candidate in topics {
                if used.insert(candidate.id.as_str()) {
                    options.push(TermExerciseOption {
                        id: candidate.id.clone(),
                        text: candidate.title.clone(),
                    });
                }
                if options.len() == 4 {
                    break;
                }
            }
            (
                "Zu welchem Themengebiet gehört dieser Begriff?",
                input.card.term.clone(),
                options,
                DistractorSource::None,
                topic.id.clone(),
            )
        }
    };

    options.sort_by(|left, right| left.id.cmp(&right.id));
    let hash_input = format!("{}|{}", input.card.term_id, direction_name(direction));
    let rotation = (fnv1a(hash_input.as_bytes()) % options.len() as u64) as usize;
    options.rotate_left(rotation);

    Some(TermExercise {
        term_id: input.card.term_id.clone(),
        direction,
        instruction: instruction.into(),
        prompt,
        options,
        distractor_source,
        cluster_title: input.card.cluster_title.clone(),
        correct_option_id,
    })
}

pub fn rotate_direction(input: &TermExerciseInput) -> Option<TermDirection> {
    let directions = available_directions(input);
    if directions.is_empty() {
        return None;
    }
    let index = input.attempt_count.rem_euclid(directions.len() as i64) as usize;
    Some(directions[index])
}

fn term_distractors(input: &TermExerciseInput) -> (Vec<TermCandidate>, DistractorSource) {
    let groups = [
        (&input.cluster_mates, DistractorSource::Cluster),
        (&input.see_also, DistractorSource::SeeAlso),
        (&input.chapter_mates, DistractorSource::Chapter),
    ];
    let mut used = HashSet::from([input.card.term_id.as_str()]);
    let mut distractors = Vec::with_capacity(3);
    let mut source = DistractorSource::None;

    'groups: for (candidates, candidate_source) in groups {
        let mut sorted = candidates.iter().collect::<Vec<_>>();
        sorted.sort_by(|left, right| left.id.cmp(&right.id));
        for candidate in sorted {
            if used.insert(candidate.id.as_str()) {
                if distractors.is_empty() {
                    source = candidate_source;
                }
                distractors.push(candidate.clone());
            }
            if distractors.len() == 3 {
                break 'groups;
            }
        }
    }

    (distractors, source)
}

fn direction_name(direction: TermDirection) -> &'static str {
    match direction {
        TermDirection::TermToDefinition => "term_to_definition",
        TermDirection::DefinitionToTerm => "definition_to_term",
        TermDirection::ScenarioToTerm => "scenario_to_term",
        TermDirection::TermToTopic => "term_to_topic",
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn candidate(id: &str) -> TermCandidate {
        TermCandidate {
            id: id.into(),
            term: format!("Begriff {id}"),
            definition: format!("Definition {id}"),
        }
    }

    fn input() -> TermExerciseInput {
        TermExerciseInput {
            card: TermCard {
                term_id: "term-main".into(),
                term: "Hauptbegriff".into(),
                definition: "Hauptdefinition".into(),
                topic: Some(TermTopic {
                    id: "topic-main".into(),
                    title: "Hauptthema".into(),
                }),
                cluster_title: Some("Ähnliche Begriffe".into()),
                distinction: Some("Dies ist der Hauptbegriff.".into()),
            },
            cluster_mates: vec![candidate("cluster-b")],
            see_also: vec![candidate("see-c")],
            chapter_mates: vec![candidate("chapter-d"), candidate("chapter-a")],
            topics: vec![
                TermTopic {
                    id: "topic-main".into(),
                    title: "Hauptthema".into(),
                },
                TermTopic {
                    id: "topic-other".into(),
                    title: "Anderes Thema".into(),
                },
            ],
            attempt_count: 0,
        }
    }

    #[test]
    fn distractor_groups_have_fixed_priority() {
        let exercise = build_exercise(&input(), TermDirection::TermToDefinition).unwrap();
        let ids = exercise
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(exercise.distractor_source, DistractorSource::Cluster);
        assert!(ids.contains("cluster-b"));
        assert!(ids.contains("see-c"));
        assert!(ids.contains("chapter-a"));
        assert!(!ids.contains("chapter-d"));
    }

    #[test]
    fn see_also_is_used_when_cluster_candidates_are_missing() {
        let mut input = input();
        input.cluster_mates.clear();
        let exercise = build_exercise(&input, TermDirection::DefinitionToTerm).unwrap();

        assert_eq!(exercise.distractor_source, DistractorSource::SeeAlso);
        assert!(exercise.options.iter().any(|option| option.id == "see-c"));
    }

    #[test]
    fn correct_option_occurs_exactly_once() {
        let mut input = input();
        input.cluster_mates.push(candidate("term-main"));
        input.see_also.push(candidate("cluster-b"));
        let exercise = build_exercise(&input, TermDirection::TermToDefinition).unwrap();

        assert_eq!(exercise.correct_option_id, "term-main");
        assert_eq!(
            exercise
                .options
                .iter()
                .filter(|option| option.id == exercise.correct_option_id)
                .count(),
            1
        );
    }

    #[test]
    fn option_order_is_reproducible() {
        let input = input();
        let first = build_exercise(&input, TermDirection::TermToDefinition).unwrap();
        let second = build_exercise(&input, TermDirection::TermToDefinition).unwrap();

        assert_eq!(
            first
                .options
                .iter()
                .map(|option| option.id.as_str())
                .collect::<Vec<_>>(),
            second
                .options
                .iter()
                .map(|option| option.id.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn correct_option_position_varies_between_terms() {
        let mut positions = HashSet::new();
        for index in 0..8 {
            let mut input = input();
            input.card.term_id = format!("term-{index}");
            input.cluster_mates = vec![candidate("distractor")];
            input.see_also.clear();
            input.chapter_mates.clear();
            let exercise = build_exercise(&input, TermDirection::TermToDefinition).unwrap();
            positions.insert(
                exercise
                    .options
                    .iter()
                    .position(|option| option.id == exercise.correct_option_id)
                    .unwrap(),
            );
        }

        assert!(positions.len() >= 2);
    }

    #[test]
    fn scenario_requires_a_distinction() {
        let mut input = input();
        input.card.distinction = None;

        assert!(!available_directions(&input).contains(&TermDirection::ScenarioToTerm));
        assert!(build_exercise(&input, TermDirection::ScenarioToTerm).is_none());
    }

    #[test]
    fn one_candidate_creates_two_options() {
        let mut input = input();
        input.see_also.clear();
        input.chapter_mates.clear();
        let exercise = build_exercise(&input, TermDirection::DefinitionToTerm).unwrap();

        assert_eq!(exercise.options.len(), 2);
    }

    #[test]
    fn direction_rotates_with_attempt_count() {
        let mut input = input();
        let expected = [
            TermDirection::TermToDefinition,
            TermDirection::DefinitionToTerm,
            TermDirection::ScenarioToTerm,
            TermDirection::TermToTopic,
        ];

        for (attempt_count, direction) in expected.into_iter().enumerate() {
            input.attempt_count = attempt_count as i64;
            assert_eq!(rotate_direction(&input), Some(direction));
        }
    }
}
