-- Subscription Tokens 테이블을 생성한다.
CREATE TABLE subscription_tokens (
    subscription_token TEXT NOT NULL PRIMARY KEY,
    subscriber_id UUID NOT NULL REFERENCES subscriptions (id)
);