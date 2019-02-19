table! {
    objects (id) {
        id -> Text,
        created_by -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    properties (id) {
        id -> Int8,
        created_by -> Int8,
        created_at -> Timestamp,
        display -> Text,
        #[sql_name = "type"]
        type_ -> Text,
    }
}

table! {
    property_select_choices (id, property_id) {
        id -> Int4,
        property_id -> Int8,
        display -> Text,
        created_by -> Int8,
        created_at -> Timestamp,
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

table! {
    values (object_id, property_id) {
        object_id -> Text,
        property_id -> Int8,
        created_by -> Int8,
        created_at -> Timestamp,
        value -> Text,
    }
}

joinable!(objects -> users (created_by));
joinable!(properties -> users (created_by));
joinable!(property_select_choices -> properties (property_id));
joinable!(property_select_choices -> users (created_by));
joinable!(values -> objects (object_id));
joinable!(values -> properties (property_id));
joinable!(values -> users (created_by));

allow_tables_to_appear_in_same_query!(
    objects,
    properties,
    property_select_choices,
    users,
    values,
);
