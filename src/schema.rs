// @generated automatically by Diesel CLI.

diesel::table! {
    session (id) {
        id -> Integer,
        expires_at -> Timestamp,
        token -> Text,
        created_at -> Timestamp,
        user_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        salt -> Text,
        password_hash -> Text,
        email -> Text,
        first_name -> Text,
        last_name -> Text,
    }
}

diesel::joinable!(session -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(session, users,);
