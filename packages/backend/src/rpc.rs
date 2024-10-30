use http::StatusCode;
use qubit::{handler, Extensions, FromRequestExtensions, Router, RpcError};
use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;
use uuid::Uuid;

use super::app::{AppCtx, AppError, AppState};
use super::document as doc;

#[handler(mutation)]
async fn new_ref(ctx: AppCtx, content: Value) -> RpcResult<Uuid> {
    doc::new_ref(ctx, content).await.into()
}

#[handler(query)]
async fn head_snapshot(ctx: AppCtx, ref_id: Uuid) -> RpcResult<Value> {
    doc::head_snapshot(ctx, ref_id).await.into()
}

#[handler(mutation)]
async fn save_snapshot(ctx: AppCtx, data: doc::RefContent) -> RpcResult<()> {
    doc::save_snapshot(ctx, data).await.into()
}

#[handler(query)]
async fn doc_id(ctx: AppCtx, ref_id: Uuid) -> RpcResult<String> {
    doc::doc_id(ctx, ref_id).await.into()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .handler(new_ref)
        .handler(head_snapshot)
        .handler(save_snapshot)
        .handler(doc_id)
}

/// Result returned by an RPC handler.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "tag")]
enum RpcResult<T> {
    Ok { content: T },
    Err { code: u16, message: String },
}

impl<T> From<AppError> for RpcResult<T> {
    fn from(error: AppError) -> Self {
        let code = match error {
            AppError::Db(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        RpcResult::Err {
            code: code.as_u16(),
            message: error.to_string(),
        }
    }
}

impl<T> From<Result<T, AppError>> for RpcResult<T> {
    fn from(result: Result<T, AppError>) -> Self {
        match result {
            Ok(content) => RpcResult::Ok { content },
            Err(error) => error.into(),
        }
    }
}

/// Extract user from request extension, if present.
impl FromRequestExtensions<AppState> for AppCtx {
    async fn from_request_extensions(
        state: AppState,
        mut extensions: Extensions,
    ) -> Result<Self, RpcError> {
        Ok(AppCtx {
            state,
            user: extensions.remove(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn rspc_type_defs() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pkg").join("src");
        super::router().write_bindings_to_dir(dir);
    }
}
