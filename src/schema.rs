diesel::table! {
    repos (full_path) {
        created_at -> Timestamp,
        updated_at -> Timestamp,
        host -> Text,
        repo -> Text,
        owner -> Text,
        remote_url -> Text,
        base_dir -> Text,
        full_path -> Text,
    }
}
