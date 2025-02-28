use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub password_hash: String,
    pub balance_cents: i32,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub is_admin: bool,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User, foreign_key = seller_id))]
#[diesel(table_name = crate::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub price_cents: i32,
    pub amount: i32,
    pub seller_id: i32,
    pub created_at: chrono::DateTime<chrono::Local>,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User, foreign_key = uploader_id))]
#[diesel(belongs_to(Item, foreign_key = item_id))]
#[diesel(table_name = crate::schema::attachments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct Attachment {
    pub id: i32,
    pub file_path: String,
    pub thumbnail_path: String,
    pub item_id: Option<i32>,
    pub uploader_id: i32,
    pub uploaded_at: chrono::DateTime<chrono::Local>,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Item, foreign_key = item_id))]
#[diesel(belongs_to(User, foreign_key = receiver_id))]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub id: i32,
    pub item_id: Option<i32>,
    pub payer_id: Option<i32>,
    pub receiver_id: i32,
    pub item_amount: i32,
    pub transacted_at: chrono::DateTime<chrono::Local>,
}
