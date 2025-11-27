-- migrations/003_create_tags.sql
CREATE TABLE IF NOT EXISTS tags (
    id BIGSERIAL PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Drop existing indexes if they exist and recreate with IF NOT EXISTS
DROP INDEX IF EXISTS idx_tags_organization_id;
DROP INDEX IF EXISTS idx_tags_organization_name;

CREATE INDEX IF NOT EXISTS idx_tags_organization_id ON tags(organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_organization_name ON tags(organization_id, LOWER(name));

-- Create the function if it doesn't exist
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Drop existing trigger if it exists and recreate
DROP TRIGGER IF EXISTS update_tags_updated_at ON tags;

CREATE TRIGGER update_tags_updated_at 
    BEFORE UPDATE ON tags 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();