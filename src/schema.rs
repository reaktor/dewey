table! {
    choice_values (object_id, property_id, value_id) {
        object_id -> Text,
        property_id -> Int8,
        value_id -> Int8,
        created_by -> Int8,
        created_at -> Timestamptz,
    }
}

table! {
    objects (id) {
        id -> Text,
        created_by -> Int8,
        created_at -> Timestamptz,
        extension -> Text,
    }
}

table! {
    properties (id) {
        id -> Int8,
        created_by -> Int8,
        created_at -> Timestamptz,
        ord -> Float4,
        display -> Text,
        property_type -> Property_type,
    }
}

table! {
    property_value_choices (id) {
        id -> Int8,
        property_id -> Int8,
        display -> Text,
        created_by -> Int8,
        created_at -> Timestamptz,
    }
}

table! {
    relation_values (object_id, property_id, target_id) {
        object_id -> Text,
        property_id -> Int8,
        target_id -> Text,
        created_by -> Int8,
        created_at -> Timestamptz,
    }
}

table! {
    text_values (object_id, property_id) {
        object_id -> Text,
        property_id -> Int8,
        value -> Text,
        created_by -> Int8,
        created_at -> Timestamptz,
    }
}

table! {
    timestamptz_values (object_id, property_id) {
        object_id -> Text,
        property_id -> Int8,
        value -> Nullable<Timestamptz>,
        created_by -> Int8,
        created_at -> Timestamptz,
    }
}

table! {
    user_tokens (user_id) {
        user_id -> Int8,
        google_resource_id -> Text,
        version -> Int4,
        created_at -> Timestamptz,
        access_token -> Text,
        refresh_token -> Text,
        token_expiration -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int8,
        google_resource_id -> Nullable<Text>,
        full_name -> Text,
        display_name -> Text,
    }
}

joinable!(choice_values -> objects (object_id));
joinable!(choice_values -> properties (property_id));
joinable!(choice_values -> property_value_choices (value_id));
joinable!(choice_values -> users (created_by));
joinable!(objects -> users (created_by));
joinable!(properties -> users (created_by));
joinable!(property_value_choices -> properties (property_id));
joinable!(property_value_choices -> users (created_by));
joinable!(relation_values -> properties (property_id));
joinable!(relation_values -> users (created_by));
joinable!(text_values -> objects (object_id));
joinable!(text_values -> properties (property_id));
joinable!(text_values -> users (created_by));
joinable!(timestamptz_values -> objects (object_id));
joinable!(timestamptz_values -> properties (property_id));
joinable!(timestamptz_values -> users (created_by));
joinable!(user_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    choice_values,
    objects,
    properties,
    property_value_choices,
    relation_values,
    text_values,
    timestamptz_values,
    user_tokens,
    users,
);
