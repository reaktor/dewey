CREATE VIEW view_user_id_token_versions AS
    SELECT user_id, users.google_resource_id, version
	FROM users INNER JOIN user_tokens ON (users.google_resource_id = user_tokens.google_resource_id);
