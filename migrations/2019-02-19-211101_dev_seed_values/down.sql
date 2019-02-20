
DELETE FROM objects
WHERE created_by = 901 OR
      created_by = 902 OR
      created_by = 903;

DELETE FROM properties
WHERE id = 920;

DELETE FROM users
WHERE id = 901 OR
      id = 902 OR
      id = 903;
