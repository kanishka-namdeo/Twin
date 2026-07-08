-- FTS5 virtual table for full-text search on transcripts
CREATE VIRTUAL TABLE IF NOT EXISTS transcripts_fts USING fts5(
    transcript,
    content='transcripts',
    content_rowid='rowid'
);

-- Trigger to keep FTS index in sync with transcripts table
CREATE TRIGGER IF NOT EXISTS transcripts_ai AFTER INSERT ON transcripts BEGIN
  INSERT INTO transcripts_fts(rowid, transcript) VALUES (new.rowid, new.transcript);
END;

CREATE TRIGGER IF NOT EXISTS transcripts_ad AFTER DELETE ON transcripts BEGIN
  INSERT INTO transcripts_fts(transcripts_fts, rowid, transcript) VALUES('delete', old.rowid, old.transcript);
END;

CREATE TRIGGER IF NOT EXISTS transcripts_au AFTER UPDATE ON transcripts BEGIN
  INSERT INTO transcripts_fts(transcripts_fts, rowid, transcript) VALUES('delete', old.rowid, old.transcript);
  INSERT INTO transcripts_fts(rowid, transcript) VALUES (new.rowid, new.transcript);
END;
