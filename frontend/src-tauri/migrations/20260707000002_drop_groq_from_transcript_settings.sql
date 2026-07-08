-- Migration: Remove deprecated groqApiKey column from transcript_settings table
-- Keeps: whisperApiKey, deepgramApiKey, elevenLabsApiKey, openaiApiKey

PRAGMA foreign_keys=off;

-- Create new transcript_settings table without groqApiKey column
CREATE TABLE IF NOT EXISTS transcript_settings_new (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    whisperApiKey TEXT,
    deepgramApiKey TEXT,
    elevenLabsApiKey TEXT,
    openaiApiKey TEXT
);

-- Copy data from old table to new table (only keeping columns we want)
INSERT INTO transcript_settings_new (id, provider, model, whisperApiKey, deepgramApiKey, elevenLabsApiKey, openaiApiKey)
SELECT id, provider, model, whisperApiKey, deepgramApiKey, elevenLabsApiKey, openaiApiKey
FROM transcript_settings;

-- Drop the old table
DROP TABLE transcript_settings;

-- Rename new table to original name
ALTER TABLE transcript_settings_new RENAME TO transcript_settings;

PRAGMA foreign_keys=on;
