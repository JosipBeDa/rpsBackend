table! {
    messages (id) {
        id -> Int4,
        sender_id -> Varchar,
        rec_user -> Nullable<Varchar>,
        rec_room -> Nullable<Varchar>,
        body -> Nullable<Varchar>,
        time_sent -> Nullable<Timestamptz>,
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
    user_room_junction (id) {
        id -> Int4,
        user_id -> Varchar,
        room_id -> Nullable<Varchar>,
        user_is_admin -> Bool,
    }
}

table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        password -> Varchar,
    }
}

joinable!(messages -> rooms (rec_room));
joinable!(rooms -> users (admin));
joinable!(user_room_junction -> rooms (room_id));
joinable!(user_room_junction -> users (user_id));

allow_tables_to_appear_in_same_query!(
    messages,
    rooms,
    user_room_junction,
    users,
);
