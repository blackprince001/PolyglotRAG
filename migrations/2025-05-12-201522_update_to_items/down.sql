-- This file should undo anything in `up.sql`
ALTER TABLE embeddings RENAME COLUMN content_chunk_id TO chunk_id;