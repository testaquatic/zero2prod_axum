-- Add up migration script here
ALTER TABLE users RENAME COLUMN hashed_password TO password_hash;
