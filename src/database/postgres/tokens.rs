use moka::future::Cache;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::domain::credential::Claims;

#[tracing::instrument(name = "Save token", skip_all, err(Debug))]
pub async fn save_token(
    pg_excutor: impl PgExecutor<'_>,
    moka_cache: &Cache<Uuid, Uuid>,
    claims: &Claims,
    user_id: &Uuid,
) -> Result<(), sqlx::Error> {
    moka_cache.insert(claims.auth_id, *user_id).await;

    sqlx::query!(
        r#"
        INSERT INTO auth_tokens (auth_id, user_id, exp, iat)
        VALUES ($1, $2, $3, $4);
        "#,
        claims.auth_id,
        user_id,
        claims.exp,
        claims.iat,
    )
    .execute(pg_excutor)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Get user_id by token_id", skip_all, err(Debug))]
pub async fn get_user_id_by_token_id(
    pg_excutor: impl PgExecutor<'_>,
    moka_cache: &Cache<Uuid, Uuid>,
    token_id: &Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    if let Some(user_id) = moka_cache.get(token_id).await {
        return Ok(Some(user_id));
    }

    let result = sqlx::query!(
        r#"
        SELECT user_id
        FROM auth_tokens
        WHERE auth_id = $1
        "#,
        token_id,
    )
    .fetch_optional(pg_excutor)
    .await?;

    if let Some(row) = result {
        moka_cache.insert(*token_id, row.user_id).await;
        Ok(Some(row.user_id))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(name = "Delete expired tokens", skip_all, err(Debug))]
pub async fn delete_expired_token(
    pg_excutor: impl PgExecutor<'_>,
    date: &chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM auth_tokens
        WHERE exp < $1;
        "#,
        date.timestamp(),
    )
    .execute(pg_excutor)
    .await?;

    Ok(())
}
