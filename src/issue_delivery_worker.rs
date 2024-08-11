use tracing::field::display;

use crate::{
    database::PostgresPool, domain::SubscriberEmail, email_client::Postmark, error::Z2PAError,
    settings::Settings,
};

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(skip_all, fields(newsletter_issue_id = tracing::field::Empty, subscriber_email = tracing::field::Empty), err)]
pub async fn try_excute_task(
    pool: &PostgresPool,
    email_client: &Postmark,
) -> Result<ExecutionOutcome, Z2PAError> {
    match pool.dequeue_task().await? {
        None => Ok(ExecutionOutcome::EmptyQueue),
        Some((transaction, issue_id, email)) => {
            tracing::Span::current()
                .record("newsletter_issue_id", display(&issue_id))
                .record("subscriber_email", display(&email));

            // 이메일을 전송한다.
            match SubscriberEmail::try_from(email.clone()) {
                Ok(email) => {
                    let issue = pool.get_issue(issue_id).await?;
                    if let Err(e) = email_client
                        .send_email(
                            &email,
                            &issue.title,
                            &issue.html_content,
                            &issue.text_content,
                        )
                        .await
                    {
                        tracing::error!(
                            error.cause_chain = ?e,
                            error.message = %e,
                            "Failed to deliever issue to a confirmed subscriber. Skipping.",
                        )
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error.cause_chain = ?e,
                        error.message = %e,
                        "Skipping a confirmed subscriber. Their stored contact details are invalid",
                    );
                }
            }
            transaction.delete_task(issue_id, &email).await?;

            Ok(ExecutionOutcome::TaskCompleted)
        }
    }
}

async fn worker_loop(pool: &PostgresPool, email_client: &Postmark) -> Result<(), Z2PAError> {
    loop {
        match try_excute_task(pool, email_client).await {
            Ok(ExecutionOutcome::TaskCompleted) => (),
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn run_worker_until_stopped(settings: Settings) -> Result<(), anyhow::Error> {
    let pool = PostgresPool::connect(settings.database.connect_options_with_db())?;
    let email_client = settings.email_client.get_email_client()?;

    worker_loop(&pool, &email_client).await?;

    Ok(())
}
