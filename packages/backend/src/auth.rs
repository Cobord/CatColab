use firebase_auth::{FirebaseAuth, FirebaseUser};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::app::{AppCtx, AppError, AppState};

/// Levels of permission that a user can have on a document.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, sqlx::Type, TS,
)]
#[sqlx(type_name = "permission_level", rename_all = "lowercase")]
pub enum PermissionLevel {
    Read,
    Write,
    Maintain,
    Own,
}

/// Global and user permission levels on a document.
#[derive(Clone, Debug, Deserialize, Serialize, TS)]
pub struct Permissions {
    anyone: Option<PermissionLevel>,
    user: Option<PermissionLevel>,
}

impl Permissions {
    /// Gets the highest level of permissions allowed.
    pub fn max_level(self) -> Option<PermissionLevel> {
        self.anyone.into_iter().chain(self.user).reduce(std::cmp::max)
    }
}

/** Verify that user is authorized to access a ref at a given permission level.

It is safe to proceed if the result is `Ok`; otherwise, the requested action
should be aborted.
 */
pub async fn authorize(ctx: &AppCtx, ref_id: Uuid, level: PermissionLevel) -> Result<(), AppError> {
    let authorized = is_authorized(ctx, ref_id, level).await?;
    if authorized {
        Ok(())
    } else {
        Err(AppError::Forbidden(ref_id))
    }
}

/** Is the user authorized to access a ref at a given permission level?

The result is an error if the ref does not exist.
 */
pub async fn is_authorized(
    ctx: &AppCtx,
    ref_id: Uuid,
    level: PermissionLevel,
) -> Result<bool, AppError> {
    match max_permission_level(ctx, ref_id).await? {
        Some(max_level) => Ok(level <= max_level),
        None => Ok(false),
    }
}

/// Gets the highest level of permissions allowed for a ref.
pub async fn max_permission_level(
    ctx: &AppCtx,
    ref_id: Uuid,
) -> Result<Option<PermissionLevel>, AppError> {
    let query = sqlx::query_scalar!(
        r#"
        SELECT MAX(level) AS "max: PermissionLevel" FROM permissions
        WHERE object = $1 AND (subject IS NULL OR subject = $2)
        "#,
        ref_id,
        ctx.user.as_ref().map(|user| user.user_id.clone())
    );
    let level = query.fetch_one(&ctx.state.db).await?;

    if level.is_none() {
        ref_exists(ctx, ref_id).await?;
    }

    Ok(level)
}

/// Gets the permissions allowed for a ref.
pub async fn permissions(ctx: &AppCtx, ref_id: Uuid) -> Result<Permissions, AppError> {
    let query = sqlx::query_scalar!(
        r#"
        SELECT level as "level: PermissionLevel" FROM permissions
        WHERE object = $1 and subject iS NULL
        "#,
        ref_id
    );
    let anyone = query.fetch_optional(&ctx.state.db).await?;

    let query = sqlx::query_scalar!(
        r#"
        SELECT level as "level: PermissionLevel" FROM permissions
        WHERE object = $1 and subject = $2
        "#,
        ref_id,
        ctx.user.as_ref().map(|user| user.user_id.clone())
    );
    let user = query.fetch_optional(&ctx.state.db).await?;

    if anyone.is_none() && user.is_none() {
        ref_exists(ctx, ref_id).await?;
    }

    Ok(Permissions { anyone, user })
}

/// Inserts or updates permissions for a ref-user pair.
pub async fn upsert_permission(
    state: &AppState,
    ref_id: Uuid,
    user_id: Option<String>,
    level: PermissionLevel,
) -> Result<(), AppError> {
    let query = sqlx::query!(
        "
        INSERT INTO permissions(object, subject, level)
        VALUES ($1, $2, $3)
        ON CONFLICT(object, subject)
        DO UPDATE SET level = EXCLUDED.level;
        ",
        ref_id,
        user_id,
        level as PermissionLevel,
    );
    query.execute(&state.db).await?;
    Ok(())
}

/// Verify that the given ref exists.
async fn ref_exists(ctx: &AppCtx, ref_id: Uuid) -> Result<(), AppError> {
    let query = sqlx::query_scalar!("SELECT 1 FROM refs WHERE id = $1", ref_id);
    query.fetch_one(&ctx.state.db).await?;
    Ok(())
}

/** Extracts an authenticated user from an HTTP request.

Note that the `firebase_auth` crate has an Axum feature with similar
functionality, but we don't use it because it doesn't integrate well with the
RPC service.
 */
pub fn authenticate_from_request<T>(
    firebase_auth: &FirebaseAuth,
    req: &hyper::Request<T>,
) -> Result<Option<FirebaseUser>, String> {
    let maybe_auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    maybe_auth_header
        .map(|auth_header| {
            let bearer = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| "Missing Bearer token".to_string())?;

            firebase_auth
                .verify(bearer)
                .map_err(|err| format!("Failed to verify token: {}", err))
        })
        .transpose()
}
