-- Add meeting_context field to meetings table for storing context/purpose
ALTER TABLE meetings ADD COLUMN meeting_context TEXT;
