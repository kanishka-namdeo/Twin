-- Migration: Remove deprecated LLM provider columns from settings table
-- Removes: groqApiKey, openRouterApiKey, geminiApiKey
-- Keeps: openaiApiKey, anthropicApiKey, ollamaApiKey, ollamaEndpoint, customOpenAIConfig

-- SQLite doesn't support DROP COLUMN directly in older versions, so we recreate the table
PRAGMA foreign_keys=off;

-- Create new settings table without deprecated columns
CREATE TABLE IF NOT EXISTS settings_new (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    whisperModel TEXT NOT NULL,
    openaiApiKey TEXT,
    anthropicApiKey TEXT,
    ollamaApiKey TEXT,
    ollamaEndpoint TEXT,
    customOpenAIConfig TEXT
);

-- Copy data from old table to new table (only keeping columns we want)
INSERT INTO settings_new (id, provider, model, whisperModel, openaiApiKey, anthropicApiKey, ollamaApiKey, ollamaEndpoint, customOpenAIConfig)
SELECT id, provider, model, whisperModel, openaiApiKey, anthropicApiKey, ollamaApiKey, ollamaEndpoint, customOpenAIConfig
FROM settings;

-- Drop the old table
DROP TABLE settings;

-- Rename new table to original name
ALTER TABLE settings_new RENAME TO settings;

PRAGMA foreign_keys=on;
