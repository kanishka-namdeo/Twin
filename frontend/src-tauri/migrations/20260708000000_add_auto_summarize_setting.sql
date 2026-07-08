-- Add autoSummarizeEnabled column to settings table
ALTER TABLE settings ADD COLUMN autoSummarizeEnabled INTEGER NOT NULL DEFAULT 0;
