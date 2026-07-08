-- Migration: Add speaker_id column for diarization support
-- This adds a speaker_id column to transcripts table to store which speaker said each segment
-- The speaker_id references the speakers table

ALTER TABLE transcripts ADD COLUMN speaker_id INTEGER;
