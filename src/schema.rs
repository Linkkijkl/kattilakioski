// @generated automatically by Diesel CLI.

diesel::table! {
    items (id) {
        id -> Int4,
        title -> Varchar,
        description -> Varchar,
        price_cents -> Int4,
        amount -> Int4,
        seller_id -> Int4,
        created_at -> Timestamptz,
    }
}

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

diesel::joinable!(items -> users (seller_id));

diesel::allow_tables_to_appear_in_same_query!(
    items,
    users,
);
