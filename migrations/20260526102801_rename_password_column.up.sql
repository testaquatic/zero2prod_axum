-- Add up migration script here
ALTER TABLE users RENAME COLUMN password TO hashed_password;