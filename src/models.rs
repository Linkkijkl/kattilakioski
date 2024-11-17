use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub balance_cents: i32,
    pub created_at: chrono::DateTime<chrono::Local>,
}
