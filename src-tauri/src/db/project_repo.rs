use crate::db::DbError;
use crate::domain::{
    compute_text_stats, CreateDocumentInput, CreateProjectInput, DocumentDetail,
    DocumentTreeItem, FtsSearchHit, ProjectDetail, ProjectSummary, ReorderItem,
    UpdateDocumentInput,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn register_project_in_app_db(
    app_conn: &Connection,
    project: &ProjectDetail,
    db_path: &str,
) -> Result<(), DbError> {
    app_conn.execute(
        r#"
        INSERT INTO app_projects_index (id, title, language, genre, synopsis, db_path, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            language = excluded.language,
            genre = excluded.genre,
            synopsis = excluded.synopsis,
            db_path = excluded.db_path,
            updated_at = excluded.updated_at
        "#,
        params![
            project.id,
            project.title,
            project.language,
            project.genre,
            project.synopsis,
            db_path,
            project.updated_at
        ],
    )?;
    Ok(())
}

pub fn list_projects_from_app_db(app_conn: &Connection) -> Result<Vec<ProjectSummary>, DbError> {
    let mut stmt = app_conn.prepare(
        "SELECT id, title, language, genre, synopsis, db_path, updated_at FROM app_projects_index ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProjectSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            language: row.get(2)?,
            genre: row.get(3)?,
            synopsis: row.get(4)?,
            path: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub fn create_project(
    proj_conn: &Connection,
    input: CreateProjectInput,
) -> Result<ProjectDetail, DbError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let project = ProjectDetail {
        id: id.clone(),
        title: input.title,
        language: if input.language.is_empty() {
            "ko".to_string()
        } else {
            input.language
        },
        genre: input.genre,
        synopsis: input.synopsis,
        style_guide_json: None,
        daily_goal_count: 0,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    proj_conn.execute(
        r#"
        INSERT INTO projects (id, title, language, genre, synopsis, style_guide_json, daily_goal_count, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            project.id,
            project.title,
            project.language,
            project.genre,
            project.synopsis,
            project.style_guide_json,
            project.daily_goal_count,
            project.created_at,
            project.updated_at
        ],
    )?;

    // 기본 제1장 및 첫 장면 자동 생성
    let chapter = create_document(
        proj_conn,
        &project.id,
        CreateDocumentInput {
            parent_id: None,
            doc_type: "chapter".to_string(),
            title: if project.language == "en" {
                "Chapter 1".to_string()
            } else {
                "제1장".to_string()
            },
        },
    )?;

    create_document(
        proj_conn,
        &project.id,
        CreateDocumentInput {
            parent_id: Some(chapter.id),
            doc_type: "scene".to_string(),
            title: if project.language == "en" {
                "Scene 1".to_string()
            } else {
                "장면 1".to_string()
            },
        },
    )?;

    Ok(project)
}

pub fn get_project_detail(proj_conn: &Connection) -> Result<ProjectDetail, DbError> {
    proj_conn
        .query_row(
            "SELECT id, title, language, genre, synopsis, style_guide_json, daily_goal_count, created_at, updated_at FROM projects LIMIT 1",
            [],
            |row| {
                Ok(ProjectDetail {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    language: row.get(2)?,
                    genre: row.get(3)?,
                    synopsis: row.get(4)?,
                    style_guide_json: row.get(5)?,
                    daily_goal_count: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| DbError::NotFound("project".to_string()))
}

pub fn list_documents(
    proj_conn: &Connection,
    project_id: &str,
) -> Result<Vec<DocumentTreeItem>, DbError> {
    let mut stmt = proj_conn.prepare(
        r#"
        SELECT id, project_id, parent_id, doc_type, title, sort_order, status,
               revision, chars_with_space, chars_no_space, word_count, is_trashed, updated_at
        FROM documents
        WHERE project_id = ?1
        ORDER BY sort_order ASC, updated_at ASC
        "#,
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        let trashed_int: i64 = row.get(11)?;
        Ok(DocumentTreeItem {
            id: row.get(0)?,
            project_id: row.get(1)?,
            parent_id: row.get(2)?,
            doc_type: row.get(3)?,
            title: row.get(4)?,
            sort_order: row.get(5)?,
            status: row.get(6)?,
            revision: row.get(7)?,
            chars_with_space: row.get(8)?,
            chars_no_space: row.get(9)?,
            word_count: row.get(10)?,
            is_trashed: trashed_int != 0,
            updated_at: row.get(12)?,
        })
    })?;
    let mut items = Vec::new();
    for item in rows {
        items.push(item?);
    }
    Ok(items)
}

pub fn get_document(proj_conn: &Connection, doc_id: &str) -> Result<DocumentDetail, DbError> {
    proj_conn
        .query_row(
            r#"
            SELECT id, project_id, parent_id, doc_type, title, sort_order, status,
                   pov_character_id, location_id, timeline_label, content_json, plain_text,
                   revision, chars_with_space, chars_no_space, word_count, is_trashed, updated_at
            FROM documents
            WHERE id = ?1
            "#,
            params![doc_id],
            |row| {
                let trashed_int: i64 = row.get(16)?;
                Ok(DocumentDetail {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    doc_type: row.get(3)?,
                    title: row.get(4)?,
                    sort_order: row.get(5)?,
                    status: row.get(6)?,
                    pov_character_id: row.get(7)?,
                    location_id: row.get(8)?,
                    timeline_label: row.get(9)?,
                    content_json: row.get(10)?,
                    plain_text: row.get(11)?,
                    revision: row.get(12)?,
                    chars_with_space: row.get(13)?,
                    chars_no_space: row.get(14)?,
                    word_count: row.get(15)?,
                    is_trashed: trashed_int != 0,
                    updated_at: row.get(17)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| DbError::NotFound(doc_id.to_string()))
}

fn sync_document_fts(
    proj_conn: &Connection,
    doc_id: &str,
    title: &str,
    plain_text: &str,
    is_trashed: bool,
) -> Result<(), DbError> {
    proj_conn.execute("DELETE FROM documents_fts WHERE doc_id = ?1", params![doc_id])?;
    if !is_trashed {
        proj_conn.execute(
            "INSERT INTO documents_fts (doc_id, title, plain_text) VALUES (?1, ?2, ?3)",
            params![doc_id, title, plain_text],
        )?;
    }
    Ok(())
}

pub fn create_document(
    proj_conn: &Connection,
    project_id: &str,
    input: CreateDocumentInput,
) -> Result<DocumentDetail, DbError> {
    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let max_order: i64 = proj_conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM documents WHERE project_id = ?1 AND parent_id IS ?2",
        params![project_id, input.parent_id],
        |row| row.get(0),
    )?;
    let sort_order = max_order + 1;
    let initial_block_id = Uuid::new_v4().to_string();
    let default_content = serde_json::json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "attrs": { "blockId": initial_block_id }
            }
        ]
    })
    .to_string();

    proj_conn.execute(
        r#"
        INSERT INTO documents (
            id, project_id, parent_id, doc_type, title, sort_order, status,
            content_json, plain_text, revision, chars_with_space, chars_no_space, word_count,
            is_trashed, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'draft', ?7, '', 1, 0, 0, 0, 0, ?8)
        "#,
        params![
            id,
            project_id,
            input.parent_id,
            input.doc_type,
            input.title,
            sort_order,
            default_content,
            now
        ],
    )?;

    sync_document_fts(proj_conn, &id, &input.title, "", false)?;
    get_document(proj_conn, &id)
}

pub fn update_document(
    proj_conn: &Connection,
    input: UpdateDocumentInput,
) -> Result<DocumentDetail, DbError> {
    let current = get_document(proj_conn, &input.id)?;
    if let Some(expected) = input.expected_revision {
        if current.revision != expected {
            return Err(DbError::RevisionConflict {
                expected,
                actual: current.revision,
            });
        }
    }

    let next_title = input.title.unwrap_or(current.title);
    let next_status = input.status.unwrap_or(current.status);
    let next_content_json = input.content_json.unwrap_or(current.content_json);
    let next_plain_text = input.plain_text.unwrap_or(current.plain_text);
    let (chars_with_space, chars_no_space, word_count) = compute_text_stats(&next_plain_text);
    let next_revision = current.revision + 1;
    let now = now_iso();

    proj_conn.execute(
        r#"
        UPDATE documents
        SET title = ?2,
            status = ?3,
            content_json = ?4,
            plain_text = ?5,
            revision = ?6,
            chars_with_space = ?7,
            chars_no_space = ?8,
            word_count = ?9,
            updated_at = ?10
        WHERE id = ?1
        "#,
        params![
            current.id,
            next_title,
            next_status,
            next_content_json,
            next_plain_text,
            next_revision,
            chars_with_space,
            chars_no_space,
            word_count,
            now
        ],
    )?;

    sync_document_fts(
        proj_conn,
        &current.id,
        &next_title,
        &next_plain_text,
        current.is_trashed,
    )?;
    get_document(proj_conn, &current.id)
}

pub fn reorder_documents(proj_conn: &Connection, items: &[ReorderItem]) -> Result<(), DbError> {
    let tx = proj_conn.unchecked_transaction()?;
    for item in items {
        tx.execute(
            "UPDATE documents SET parent_id = ?2, sort_order = ?3 WHERE id = ?1",
            params![item.id, item.parent_id, item.sort_order],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn set_document_trashed(
    proj_conn: &Connection,
    doc_id: &str,
    trashed: bool,
) -> Result<(), DbError> {
    let doc = get_document(proj_conn, doc_id)?;
    let trashed_int = if trashed { 1 } else { 0 };
    proj_conn.execute(
        "UPDATE documents SET is_trashed = ?2, updated_at = ?3 WHERE id = ?1",
        params![doc_id, trashed_int, now_iso()],
    )?;
    sync_document_fts(proj_conn, doc_id, &doc.title, &doc.plain_text, trashed)?;
    Ok(())
}

pub fn search_documents_fts(
    proj_conn: &Connection,
    query: &str,
) -> Result<Vec<FtsSearchHit>, DbError> {
    let trimmed = query.trim();
    if trimmed.chars().count() < 3 {
        // trigram 토크나이저는 최소 3글자 이상에서 동작하므로 2글자 이하는 LIKE 폴백
        let pattern = format!("%{}%", trimmed);
        let mut stmt = proj_conn.prepare(
            "SELECT id, title, substr(plain_text, 1, 120) FROM documents WHERE is_trashed = 0 AND (title LIKE ?1 OR plain_text LIKE ?1) LIMIT 30",
        )?;
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(FtsSearchHit {
                doc_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
            })
        })?;
        let mut hits = Vec::new();
        for hit in rows {
            hits.push(hit?);
        }
        return Ok(hits);
    }

    let escaped = format!("\"{}\"", trimmed.replace('"', "\"\""));
    let mut stmt = proj_conn.prepare(
        r#"
        SELECT doc_id, title, snippet(documents_fts, 2, '[', ']', '...', 16)
        FROM documents_fts
        WHERE documents_fts MATCH ?1
        LIMIT 30
        "#,
    )?;
    let rows = stmt.query_map(params![escaped], |row| {
        Ok(FtsSearchHit {
            doc_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get(2)?,
        })
    })?;
    let mut hits = Vec::new();
    for hit in rows {
        hits.push(hit?);
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_project_db;

    #[test]
    fn test_project_and_document_crud_and_fts() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_project_db(&conn).unwrap();

        let proj = create_project(
            &conn,
            CreateProjectInput {
                title: "잊혀진 별의 기록".to_string(),
                language: "ko".to_string(),
                genre: Some("SF".to_string()),
                synopsis: None,
            },
        )
        .unwrap();

        let docs = list_documents(&conn, &proj.id).unwrap();
        assert_eq!(docs.len(), 2);
        let scene = docs.iter().find(|d| d.doc_type == "scene").unwrap();

        let updated = update_document(
            &conn,
            UpdateDocumentInput {
                id: scene.id.clone(),
                title: Some("관측소".to_string()),
                status: Some("writing".to_string()),
                content_json: None,
                plain_text: Some("그는 관측소 문을 조심스럽게 열었다.".to_string()),
                expected_revision: Some(1),
            },
        )
        .unwrap();
        assert_eq!(updated.revision, 2);
        assert_eq!(updated.chars_with_space, 20);
        assert_eq!(updated.word_count, 5);

        // Revision conflict guard
        let conflict = update_document(
            &conn,
            UpdateDocumentInput {
                id: scene.id.clone(),
                title: None,
                status: None,
                content_json: None,
                plain_text: Some("덮어쓰기".to_string()),
                expected_revision: Some(1),
            },
        );
        assert!(matches!(conflict, Err(DbError::RevisionConflict { .. })));

        // FTS5 trigram search
        let hits = search_documents_fts(&conn, "관측소").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doc_id, scene.id);

        // Trash removes from FTS
        set_document_trashed(&conn, &scene.id, true).unwrap();
        let hits_after_trash = search_documents_fts(&conn, "관측소").unwrap();
        assert!(hits_after_trash.is_empty());
    }
}
