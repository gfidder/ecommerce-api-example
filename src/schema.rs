// @generated automatically by Diesel CLI.

diesel::table! {
    session (id) {
        id -> Int4,
        expires_at -> Timestamptz,
        token -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        salt -> Text,
        password_hash -> Text,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        first_name -> Varchar,
        #[max_length = 255]
        last_name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(session -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(session, users,);
