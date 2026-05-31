-- Add up migration script here
CREATE TABLE issue_delivery_queue (
  newsletter_issue_id UUID NOT NULL REFERENCES newsletter_issues (newsletter_issue_id),
  subscriber_id UUID NOT NULL REFERENCES subscriptions (id),
  PRIMARY KEY (newsletter_issue_id, subscriber_id)
);