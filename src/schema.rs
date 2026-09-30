// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        hair_color -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(posts, users,);
