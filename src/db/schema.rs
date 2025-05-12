// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

    content_chunks (id) {
        id -> Uuid,
        file_id -> Nullable<Uuid>,
        chunk_text -> Text,
        chunk_index -> Int4,
        token_count -> Nullable<Int4>,
        page_number -> Nullable<Int4>,
        section_path -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

    embeddings (id) {
        id -> Uuid,
        content_chunk_id -> Nullable<Uuid>,
        embedding -> Nullable<Vector>,
        model_name -> Text,
        model_version -> Nullable<Text>,
        generated_at -> Nullable<Timestamptz>,
        generation_parameters -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

    files (id) {
        id -> Uuid,
        file_path -> Text,
        file_name -> Text,
        file_size -> Nullable<Int8>,
        file_type -> Nullable<Text>,
        file_hash -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::joinable!(content_chunks -> files (file_id));
diesel::joinable!(embeddings -> content_chunks (content_chunk_id));

diesel::allow_tables_to_appear_in_same_query!(
    content_chunks,
    embeddings,
    files,
);
