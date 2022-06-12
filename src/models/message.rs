use uuid::Uuid;
use diesel::Queryable;

#[derive(Queryable)]
pub struct Message {
    id: u64,
    sender_id: Uuid,
    room_id: Uuid,
    body: String,
    time_sent: chrono::DateTime<chrono::Local>
}