ALTER TABLE users
ADD COLUMN photo_url TEXT;

UPDATE users
SET kind = 'person',
    photo_url = 'https://via.placeholder.com/100x100'
WHERE id > 10000;

ALTER TABLE users
ADD CONSTRAINT "person needs photo_url" CHECK (check_user_person_has_set(kind, photo_url));
