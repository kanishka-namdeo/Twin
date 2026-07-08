-- Migration: Drop the licensing table
-- The licensing system has been removed as part of backend cleanup

DROP TABLE IF EXISTS licensing;
