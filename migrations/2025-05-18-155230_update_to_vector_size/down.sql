-- This file should undo anything in `up.sql`
-- Drop the existing index on the embedding column (assuming it's currently 768 dimension)
-- This is necessary before dropping the column itself
DROP INDEX IF EXISTS embeddings_embedding_vector_l2_ops;

-- Drop the current embedding column (with 768 dimension)
-- WARNING: This will permanently delete all existing embedding data in this column!
ALTER TABLE embeddings
DROP COLUMN embedding;

-- Add the embedding column back with the original dimension (1536)
ALTER TABLE embeddings
ADD COLUMN embedding VECTOR(1536);

-- Re-create the index for vector search on the embedding column (now 1536 dimension)
-- Adjust the 'lists' parameter if needed based on your data size and performance requirements
CREATE INDEX ON embeddings USING ivfflat (embedding vector_l2_ops) WITH (lists = 100);
