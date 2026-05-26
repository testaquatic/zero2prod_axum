-- Add up migration script here
CREATE TABLE users (
  user_id UUID PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  role TEXT NOT NULL
);