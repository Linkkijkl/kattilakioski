// @generated automatically by Diesel CLI.

diesel::table! {
    attachments (id) {
        id -> Int4,
        file_path -> Varchar,
        thumbnail_path -> Varchar,
        item_id -> Nullable<Int4>,
        uploader_id -> Int4,
        uploaded_at -> Timestamptz,
    }
}

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

diesel::joinable!(attachments -> items (item_id));
diesel::joinable!(attachments -> users (uploader_id));
diesel::joinable!(items -> users (seller_id));

diesel::allow_tables_to_appear_in_same_query!(
    attachments,
    items,
    users,
);
