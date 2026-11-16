use crate::db::project_repo::{get_document, update_document};
use crate::db::DbError;
use crate::domain::{DiffLine, DocumentDetail, DocumentVersionItem, ProjectStats, UpdateDocumentInput};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

pub fn create_document_version(
    proj_conn: &Connection,
    doc_id: &str,
    snapshot_type: &str,
    note: Option<String>,
) -> Result<DocumentVersionItem, DbError> {
    let doc = get_document(proj_conn, doc_id)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    proj_conn.execute(
        r#"
        INSERT INTO document_versions (id, document_id, revision, snapshot_type, content_json, plain_text, note, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            id,
            doc.id,
            doc.revision,
            snapshot_type,
            doc.content_json,
            doc.plain_text,
            note,
            now
        ],
    )?;

    Ok(DocumentVersionItem {
        id,
        document_id: doc.id,
        revision: doc.revision,
        snapshot_type: snapshot_type.to_string(),
        note,
        plain_text: doc.plain_text,
        created_at: now,
    })
}

pub fn list_document_versions(
    proj_conn: &Connection,
    doc_id: &str,
) -> Result<Vec<DocumentVersionItem>, DbError> {
    let mut stmt = proj_conn.prepare(
        r#"
        SELECT id, document_id, revision, snapshot_type, note, plain_text, created_at
        FROM document_versions
        WHERE document_id = ?1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )?;
    let rows = stmt.query_map(params![doc_id], |row| {
        Ok(DocumentVersionItem {
            id: row.get(0)?,
            document_id: row.get(1)?,
            revision: row.get(2)?,
            snapshot_type: row.get(3)?,
            note: row.get(4)?,
            plain_text: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    let mut items = Vec::new();
    for item in rows {
        items.push(item?);
    }
    Ok(items)
}

pub fn restore_document_version(
    proj_conn: &Connection,
    version_id: &str,
) -> Result<DocumentDetail, DbError> {
    let row: Option<(String, String, String)> = proj_conn
        .query_row(
            "SELECT document_id, content_json, plain_text FROM document_versions WHERE id = ?1",
            params![version_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    let (doc_id, content_json, plain_text) =
        row.ok_or_else(|| DbError::NotFound(version_id.to_string()))?;

    create_document_version(
        proj_conn,
        &doc_id,
        "pre_restore",
        Some("Auto backup before version restore".to_string()),
    )?;

    update_document(
        proj_conn,
        UpdateDocumentInput {
            id: doc_id,
            title: None,
            status: None,
            content_json: Some(content_json),
            plain_text: Some(plain_text),
            expected_revision: None,
        },
    )
}

pub fn get_project_stats(proj_conn: &Connection, project_id: &str) -> Result<ProjectStats, DbError> {
    proj_conn
        .query_row(
            r#"
            SELECT COALESCE(SUM(chars_with_space), 0),
                   COALESCE(SUM(chars_no_space), 0),
                   COALESCE(SUM(word_count), 0),
                   COUNT(*)
            FROM documents
            WHERE project_id = ?1 AND is_trashed = 0 AND doc_type = 'scene'
            "#,
            params![project_id],
            |row| {
                Ok(ProjectStats {
                    total_chars_with_space: row.get(0)?,
                    total_chars_no_space: row.get(1)?,
                    total_word_count: row.get(2)?,
                    scene_count: row.get(3)?,
                })
            },
        )
        .map_err(DbError::from)
}

/// 두 텍스트의 줄 단위 LCS 기반 Diff를 계산합니다.
pub fn compute_line_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let m = old_lines.len();
    let n = new_lines.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < m && j < n {
        if old_lines[i] == new_lines[j] {
            result.push(DiffLine {
                kind: "equal".to_string(),
                text: old_lines[i].to_string(),
            });
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            result.push(DiffLine {
                kind: "delete".to_string(),
                text: old_lines[i].to_string(),
            });
            i += 1;
        } else {
            result.push(DiffLine {
                kind: "insert".to_string(),
                text: new_lines[j].to_string(),
            });
            j += 1;
        }
    }
    while i < m {
        result.push(DiffLine {
            kind: "delete".to_string(),
            text: old_lines[i].to_string(),
        });
        i += 1;
    }
    while j < n {
        result.push(DiffLine {
            kind: "insert".to_string(),
            text: new_lines[j].to_string(),
        });
        j += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_project_db;
    use crate::db::project_repo::{create_project, list_documents};
    use crate::domain::CreateProjectInput;

    #[test]
    fn test_version_snapshot_diff_and_restore() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_project_db(&conn).unwrap();

        let proj = create_project(
            &conn,
            CreateProjectInput {
                title: "테스트 소설".to_string(),
                language: "ko".to_string(),
                genre: None,
                synopsis: None,
            },
        )
        .unwrap();
        let docs = list_documents(&conn, &proj.id).unwrap();
        let scene = docs.iter().find(|d| d.doc_type == "scene").unwrap();

        update_document(
            &conn,
            UpdateDocumentInput {
                id: scene.id.clone(),
                title: None,
                status: None,
                content_json: None,
                plain_text: Some("첫 번째 문장\n두 번째 문장".to_string()),
                expected_revision: None,
            },
        )
        .unwrap();

        let snap = create_document_version(&conn, &scene.id, "manual", Some("1차 초고".into())).unwrap();

        update_document(
            &conn,
            UpdateDocumentInput {
                id: scene.id.clone(),
                title: None,
                status: None,
                content_json: None,
                plain_text: Some("첫 번째 문장\n수정된 문장".to_string()),
                expected_revision: None,
            },
        )
        .unwrap();

        let diff = compute_line_diff(&snap.plain_text, "첫 번째 문장\n수정된 문장");
        assert_eq!(diff.len(), 3);
        assert_eq!(diff[0].kind, "equal");
        assert_eq!(diff[1].kind, "delete");
        assert_eq!(diff[2].kind, "insert");

        let restored = restore_document_version(&conn, &snap.id).unwrap();
        assert_eq!(restored.plain_text, "첫 번째 문장\n두 번째 문장");

        let stats = get_project_stats(&conn, &proj.id).unwrap();
        assert_eq!(stats.scene_count, 1);
        assert!(stats.total_chars_with_space > 0);
    }
}
