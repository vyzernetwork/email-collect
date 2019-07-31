table! {
    emails (address) {
        address -> Varchar,
        confirmed -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    emails,
);
