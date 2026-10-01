// @generated automatically by Diesel CLI.

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
