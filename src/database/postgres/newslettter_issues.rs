use sqlx::PgExecutor;
use uuid::Uuid;

use crate::domain::form_data::PostNewsletterData;

#[tracing::instrument(name = "Save newsletter issue", skip_all, err(Debug))]
pub async fn save_newsletter_issue(
    pg_excutor: impl PgExecutor<'_>,
    post_newsletter_data: &PostNewsletterData,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
        newsletter_issue_id,
        title,
        text_content,
        html_content,
        published_at
        )
        VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        post_newsletter_data.title,
        post_newsletter_data.text_content,
        post_newsletter_data.html_content,
    )
    .execute(pg_excutor)
    .await?;

    Ok(newsletter_issue_id)
}

pub struct NewsletterIssue {
    pub title: String,
    pub text_content: String,
    pub html_content: String,
}

pub async fn get_issue(
    pg_excutor: impl PgExecutor<'_>,
    newsletter_issue_id: Uuid,
) -> Result<NewsletterIssue, sqlx::Error> {
    sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        newsletter_issue_id
    )
    .fetch_one(pg_excutor)
    .await
}
