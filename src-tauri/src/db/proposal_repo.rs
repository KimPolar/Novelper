use crate::db::project_repo::{get_document, update_document};
use crate::db::version_repo::create_document_version;
use crate::db::DbError;
use crate::domain::{DocumentDetail, UpdateDocumentInput};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIProposal {
    pub id: String,
    pub document_id: String,
    pub base_revision: i64,
    pub target_block_id: String,
    pub original_text: String,
    pub proposed_text: String,
    pub rationale: Option<String>,
    pub status: String, // "pending" | "accepted" | "rejected" | "stale"
    pub is_stale: bool,
    pub created_at: String,
}

pub fn create_proposal(
    proj_conn: &Connection,
    doc_id: &str,
    target_block_id: &str,
    original_text: &str,
    proposed_text: &str,
    rationale: Option<String>,
) -> Result<AIProposal, DbError> {
    let doc = get_document(proj_conn, doc_id)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    proj_conn.execute(
        r#"
        INSERT INTO ai_proposals (
            id, document_id, base_revision, target_block_id,
            original_text, proposed_text, rationale, status, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)
        "#,
        params![
            id,
            doc_id,
            doc.revision,
            target_block_id,
            original_text,
            proposed_text,
            rationale,
            now
        ],
    )?;

    Ok(AIProposal {
        id,
        document_id: doc_id.to_string(),
        base_revision: doc.revision,
        target_block_id: target_block_id.to_string(),
        original_text: original_text.to_string(),
        proposed_text: proposed_text.to_string(),
        rationale,
        status: "pending".to_string(),
        is_stale: false,
        created_at: now,
    })
}

pub fn list_proposals(proj_conn: &Connection, doc_id: &str) -> Result<Vec<AIProposal>, DbError> {
    let doc = get_document(proj_conn, doc_id)?;
    let mut stmt = proj_conn.prepare(
        r#"
        SELECT id, document_id, base_revision, target_block_id, original_text, proposed_text, rationale, status, created_at
        FROM ai_proposals
        WHERE document_id = ?1
        ORDER BY created_at DESC
        "#,
    )?;
    let rows = stmt.query_map(params![doc_id], |row| {
        let base_rev: i64 = row.get(2)?;
        let status: String = row.get(7)?;
        let is_stale = status == "pending" && base_rev != doc.revision;
        Ok(AIProposal {
            id: row.get(0)?,
            document_id: row.get(1)?,
            base_revision: base_rev,
            target_block_id: row.get(3)?,
            original_text: row.get(4)?,
            proposed_text: row.get(5)?,
            rationale: row.get(6)?,
            status: if is_stale { "stale".to_string() } else { status },
            is_stale,
            created_at: row.get(8)?,
        })
    })?;
    let mut items = Vec::new();
    for item in rows {
        items.push(item?);
    }
    Ok(items)
}

fn replace_block_in_tiptap_json(
    content_json: &str,
    target_block_id: &str,
    new_text: &str,
) -> Option<(String, String)> {
    let mut doc: serde_json::Value = serde_json::from_str(content_json).ok()?;
    let nodes = doc.get_mut("content")?.as_array_mut()?;
    let mut replaced = false;
    let mut plain_lines = Vec::new();

    for node in nodes.iter_mut() {
        let block_id = node
            .pointer("/attrs/blockId")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if block_id == target_block_id && !target_block_id.is_empty() {
            node["content"] = serde_json::json!([{ "type": "text", "text": new_text }]);
            replaced = true;
            plain_lines.push(new_text.to_string());
        } else {
            let text = node
                .get("content")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                        .collect::<Vec<_>>()
                        .join("")
                })
                .unwrap_or_default();
            plain_lines.push(text);
        }
    }

    if !replaced {
        return None;
    }
    Some((doc.to_string(), plain_lines.join("\n")))
}

pub fn apply_proposal(proj_conn: &Connection, proposal_id: &str) -> Result<DocumentDetail, DbError> {
    let proposal: AIProposal = proj_conn
        .query_row(
            r#"
            SELECT id, document_id, base_revision, target_block_id, original_text, proposed_text, rationale, status, created_at
            FROM ai_proposals WHERE id = ?1
            "#,
            params![proposal_id],
            |row| {
                Ok(AIProposal {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    base_revision: row.get(2)?,
                    target_block_id: row.get(3)?,
                    original_text: row.get(4)?,
                    proposed_text: row.get(5)?,
                    rationale: row.get(6)?,
                    status: row.get(7)?,
                    is_stale: false,
                    created_at: row.get(8)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| DbError::NotFound(proposal_id.to_string()))?;

    let doc = get_document(proj_conn, &proposal.document_id)?;
    if doc.revision != proposal.base_revision {
        proj_conn.execute(
            "UPDATE ai_proposals SET status = 'stale' WHERE id = ?1",
            params![proposal_id],
        )?;
        return Err(DbError::RevisionConflict {
            expected: proposal.base_revision,
            actual: doc.revision,
        });
    }

    // 적용 전 자동 스냅샷 생성
    create_document_version(
        proj_conn,
        &doc.id,
        "pre_ai_apply",
        Some("Auto backup before applying AI proposal".to_string()),
    )?;

    let (next_json, next_plain) = replace_block_in_tiptap_json(
        &doc.content_json,
        &proposal.target_block_id,
        &proposal.proposed_text,
    )
    .unwrap_or_else(|| {
        let updated_plain = if doc.plain_text.contains(&proposal.original_text) {
            doc.plain_text
                .replacen(&proposal.original_text, &proposal.proposed_text, 1)
        } else {
            format!("{}\n{}", doc.plain_text, proposal.proposed_text)
        };
        (doc.content_json.clone(), updated_plain)
    });

    let updated = update_document(
        proj_conn,
        UpdateDocumentInput {
            id: doc.id,
            title: None,
            status: None,
            content_json: Some(next_json),
            plain_text: Some(next_plain),
            expected_revision: Some(doc.revision),
        },
    )?;

    proj_conn.execute(
        "UPDATE ai_proposals SET status = 'accepted' WHERE id = ?1",
        params![proposal_id],
    )?;

    Ok(updated)
}

pub fn reject_proposal(proj_conn: &Connection, proposal_id: &str) -> Result<(), DbError> {
    proj_conn.execute(
        "UPDATE ai_proposals SET status = 'rejected' WHERE id = ?1",
        params![proposal_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_project_db;
    use crate::db::project_repo::{create_project, list_documents};
    use crate::domain::CreateProjectInput;

    #[test]
    fn test_proposal_revision_guard_and_apply() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_project_db(&conn).unwrap();

        let proj = create_project(
            &conn,
            CreateProjectInput {
                title: "제안 테스트".into(),
                language: "ko".into(),
                genre: None,
                synopsis: None,
            },
        )
        .unwrap();
        let docs = list_documents(&conn, &proj.id).unwrap();
        let scene = docs.iter().find(|d| d.doc_type == "scene").unwrap();
        let doc = get_document(&conn, &scene.id).unwrap();

        let content = serde_json::json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "attrs": { "blockId": "blk-1" },
                "content": [{ "type": "text", "text": "그는 문을 열었다." }]
            }]
        })
        .to_string();

        let doc = update_document(
            &conn,
            UpdateDocumentInput {
                id: doc.id,
                title: None,
                status: None,
                content_json: Some(content),
                plain_text: Some("그는 문을 열었다.".into()),
                expected_revision: None,
            },
        )
        .unwrap();

        let prop = create_proposal(
            &conn,
            &doc.id,
            "blk-1",
            "그는 문을 열었다.",
            "그는 조심스럽게 문을 밀었다.",
            Some("묘사 강화".into()),
        )
        .unwrap();
        assert_eq!(prop.base_revision, doc.revision);

        let applied = apply_proposal(&conn, &prop.id).unwrap();
        assert_eq!(applied.plain_text, "그는 조심스럽게 문을 밀었다.");

        // Stale proposal check
        let stale_prop = create_proposal(
            &conn,
            &applied.id,
            "blk-1",
            "이전 문장",
            "새 문장",
            None,
        )
        .unwrap();
        update_document(
            &conn,
            UpdateDocumentInput {
                id: applied.id.clone(),
                title: None,
                status: None,
                content_json: None,
                plain_text: Some("작가가 직접 수정한 문장".into()),
                expected_revision: None,
            },
        )
        .unwrap();
        let err = apply_proposal(&conn, &stale_prop.id);
        assert!(matches!(err, Err(DbError::RevisionConflict { .. })));
    }
}
