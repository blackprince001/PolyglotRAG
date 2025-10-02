-- Your SQL goes here
-- Drop the existing index on the embedding column
-- This is necessary before dropping the column itself
DROP INDEX IF EXISTS embeddings_embedding_vector_l2_ops;

-- Drop the existing embedding column
-- WARNING: This will permanently delete all existing embedding data!
ALTER TABLE embeddings
DROP COLUMN embedding;

-- Add the new embedding column with the updated dimension (768)
ALTER TABLE embeddings
ADD COLUMN embedding VECTOR(1024);

-- Re-create the index for vector search on the new embedding column
-- Adjust the 'lists' parameter if needed based on your data size and performance requirements
CREATE INDEX ON embeddings USING ivfflat (embedding vector_l2_ops) WITH (lists = 100);

