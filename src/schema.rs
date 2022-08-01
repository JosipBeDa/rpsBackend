table! {
    hall_of_fame (id) {
        id -> Int4,
        user_id -> Varchar,
        score -> Int4,
    }
}

table! {
    messages (id) {
        id -> Varchar,
        sender_id -> Varchar,
        receiver_user -> Nullable<Varchar>,
        receiver_room -> Nullable<Varchar>,
        content -> Nullable<Varchar>,
        timestamp -> Nullable<Timestamptz>,
        read -> Nullable<Bool>,
    }
}

table! {
    room_connections (room_id, user_id) {
        room_id -> Varchar,
        user_id -> Varchar,
    }
}

table! {
    rooms (id) {
        id -> Varchar,
        name -> Varchar,
        password -> Nullable<Varchar>,
        admin -> Nullable<Varchar>,
    }
}

table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        password -> Varchar,
    }
}

joinable!(hall_of_fame -> users (user_id));
joinable!(messages -> rooms (receiver_room));
joinable!(room_connections -> rooms (room_id));
joinable!(room_connections -> users (user_id));
joinable!(rooms -> users (admin));

allow_tables_to_appear_in_same_query!(
    hall_of_fame,
    messages,
    room_connections,
    rooms,
    users,
);
