// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        balance_cents -> Int4,
        is_admin -> Bool,
        created_at -> Timestamptz,
    }
}
