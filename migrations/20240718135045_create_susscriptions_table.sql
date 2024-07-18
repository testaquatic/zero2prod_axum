CREATE TABLE subscriptions(
    id UUID NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    subscribed_at TIMESTAMPTZ NOT NULL
);