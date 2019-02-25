CREATE TYPE user_kind AS ENUM ('person', 'reserved', 'plugin');

ALTER TABLE users
ADD COLUMN public_email TEXT,
ADD COLUMN kind user_kind;

CREATE FUNCTION check_user_person_has_set(kind user_kind, must_be_set TEXT)
RETURNS BOOLEAN AS
$$
BEGIN
IF kind = 'person'
THEN RETURN must_be_set != NULL;
ELSE RETURN true;
END IF;
END;
$$ LANGUAGE PLpgSQL;

DELETE FROM users
WHERE id > 10000 AND google_resource_id IS NULL;

UPDATE users
SET kind = 'person',
    public_email = 'notyetset@example.com'
WHERE id > 10000;

UPDATE users
SET kind = 'reserved'
WHERE id <= 10000;

ALTER TABLE users
ALTER COLUMN kind SET NOT NULL;

ALTER TABLE users
ADD CONSTRAINT "person needs email" CHECK (check_user_person_has_set(kind, public_email)),
ADD CONSTRAINT "person needs google resource id" CHECK (check_user_person_has_set(kind, google_resource_id));
