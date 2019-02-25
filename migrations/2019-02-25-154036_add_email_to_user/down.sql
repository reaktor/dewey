ALTER TABLE users
DROP CONSTRAINT "person needs email",
DROP CONSTRAINT "person needs google resource id",
DROP COLUMN public_email,
DROP COLUMN kind;

DROP FUNCTION check_user_person_has_set(kind user_kind, must_be_set TEXT);

DROP TYPE user_kind;
