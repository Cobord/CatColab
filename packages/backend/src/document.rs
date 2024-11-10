//! Procedures to create and manipulate documents.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use uuid::Uuid;

use super::app::{AppCtx, AppError, AppState};
use super::auth::{upsert_permission, PermissionLevel, Permissions};

/// Creates a new document ref with initial content.
pub async fn new_ref(ctx: AppCtx, input: NewRef) -> Result<Uuid, AppError> {
    let ref_id = Uuid::now_v7();
    let NewRef {
        content,
        permissions,
    } = input;

    let query = sqlx::query!(
        "
        WITH snapshot AS (
            INSERT INTO snapshots(for_ref, content, last_updated)
            VALUES ($1, $2, NOW())
            RETURNING id
        )
        INSERT INTO refs(id, head, created)
        VALUES ($1, (SELECT id FROM snapshot), NOW())
        ",
        ref_id,
        content
    );
    query.execute(&ctx.state.db).await?;

    // Set initial permissions for ref.
    let user_id = ctx.user.map(|user| user.user_id);
    if user_id.is_some() {
        if let Some(anyone_level) = permissions.anyone {
            upsert_permission(&ctx.state, ref_id, None, anyone_level).await?;
        }
    }
    let user_level = permissions.user.unwrap_or(PermissionLevel::Own);
    upsert_permission(&ctx.state, ref_id, user_id, user_level).await?;

    Ok(ref_id)
}

/// Gets the content of the head snapshot for a document ref.
pub async fn head_snapshot(state: AppState, ref_id: Uuid) -> Result<Value, AppError> {
    let query = sqlx::query!(
        "
        SELECT content FROM snapshots
        WHERE id = (SELECT head FROM refs WHERE id = $1)
        ",
        ref_id
    );
    Ok(query.fetch_one(&state.db).await?.content)
}

/// Saves the document by overwriting the snapshot at the current head.
pub async fn autosave(state: AppState, data: RefContent) -> Result<(), AppError> {
    let RefContent { ref_id, content } = data;
    let query = sqlx::query!(
        "
        UPDATE snapshots
        SET content = $2, last_updated = NOW()
        WHERE id = (SELECT head FROM refs WHERE id = $1)
        ",
        ref_id,
        content
    );
    query.execute(&state.db).await?;
    Ok(())
}

/** Saves the document by replacing the head with a new snapshot.

The snapshot at the previous head is *not* deleted.
*/
pub async fn save_snapshot(state: AppState, data: RefContent) -> Result<(), AppError> {
    let RefContent { ref_id, content } = data;
    let query = sqlx::query!(
        "
        WITH snapshot AS (
            INSERT INTO snapshots(for_ref, content, last_updated)
            VALUES ($1, $2, NOW())
            RETURNING id
        )
        UPDATE refs
        SET head = (SELECT id FROM snapshot)
        WHERE id = $1
        ",
        ref_id,
        content
    );
    query.execute(&state.db).await?;
    Ok(())
}

/// Gets an Automerge document ID for the document ref.
pub async fn doc_id(state: AppState, ref_id: Uuid) -> Result<String, AppError> {
    let automerge_io = &state.automerge_io;
    let ack = automerge_io.emit_with_ack::<Vec<Option<String>>>("get_doc", ref_id).unwrap();
    let mut response = ack.await?;

    let maybe_doc_id = response.data.pop().flatten();
    if let Some(doc_id) = maybe_doc_id {
        // If an Automerge doc handle for this ref already exists, return it.
        Ok(doc_id)
    } else {
        // Otherwise, fetch the content from the database and create a new
        // Automerge doc handle.
        let content = head_snapshot(state.clone(), ref_id).await?;
        let data = RefContent { ref_id, content };
        let ack = automerge_io.emit_with_ack::<Vec<String>>("create_doc", data).unwrap();
        let response = ack.await?;
        Ok(response.data[0].to_string())
    }
}

/// Input to the [`new_ref`] procedure.
#[derive(Debug, Deserialize, TS)]
pub struct NewRef {
    pub content: Value,
    #[ts(optional, as = "Option<_>")]
    #[serde(default)]
    pub permissions: Permissions,
}

/// A document ref along with its content.
#[derive(Debug, Serialize, Deserialize, TS)]
pub struct RefContent {
    #[serde(rename = "refId")]
    pub ref_id: Uuid,
    pub content: Value,
}
