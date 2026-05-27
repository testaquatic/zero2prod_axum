-- Add up migration script here
CREATE TABLE auth_tokens (
  auth_id UUID NOT NULL,
  user_id UUID NOT NULL REFERENCES users (user_id),
  exp BIGINT NOT NULL,
  iat BIGINT NOT NULL,
  PRIMARY KEY (auth_id)
);

CREATE INDEX idx_auth_tokens_exp ON auth_tokens (exp);
