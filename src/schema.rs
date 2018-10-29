table! {
    exchanges (id) {
        id -> Uuid,
        from_currency -> Varchar,
        to_currency -> Varchar,
        amount -> Numeric,
        reserved_for -> Int4,
        rate -> Float8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        authentication_token -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}