use crate::adapter::schema;
use diesel::pg::Pg;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Insertable, Selectable, Identifiable, Debug, PartialEq, Clone)]
#[diesel(table_name = schema::subscriptions)]
#[diesel(check_for_backend(Pg))]
pub struct Subscription {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub subscribed_at: chrono::DateTime<chrono::Utc>,
}
