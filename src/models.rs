use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(serde::Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub balance_cents: i32,
    pub created_at: chrono::DateTime<chrono::Local>,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User, foreign_key = seller_id))]
#[diesel(table_name = crate::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(serde::Serialize)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub price_cents: i32,
    pub amount: i32,
    pub seller_id: i32,
    pub created_at: chrono::DateTime<chrono::Local>,
}
