ALTER TABLE attempts ADD COLUMN confidence TEXT NOT NULL DEFAULT 'unsure';
ALTER TABLE attempts ADD COLUMN next_review_at TEXT NOT NULL DEFAULT '';

ALTER TABLE question_options ADD COLUMN misconception TEXT;
UPDATE question_options
SET misconception = feedback
WHERE is_correct = 0 AND misconception IS NULL;

ALTER TABLE questions ADD COLUMN review_status TEXT NOT NULL DEFAULT 'approved';
ALTER TABLE questions ADD COLUMN difficulty TEXT NOT NULL DEFAULT 'exam';

CREATE INDEX IF NOT EXISTS idx_attempts_review
ON attempts(profile_id, next_review_at);
