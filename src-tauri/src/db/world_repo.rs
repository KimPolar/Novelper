use crate::db::DbError;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterItem {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub role: Option<String>,
    pub profile_json: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationItem {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldbuildingItem {
    pub id: String,
    pub project_id: String,
    pub category: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventItem {
    pub id: String,
    pub project_id: String,
    pub document_id: Option<String>,
    pub sequence_index: i64,
    pub in_story_time: Option<String>,
    pub title: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactItem {
    pub id: String,
    pub project_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub fact_text: String,
    pub certainty: String, // "confirmed" | "inferred" | "conflict"
    pub evidence_doc_id: Option<String>,
    pub evidence_block_id: Option<String>,
    pub evidence_quote: Option<String>,
    pub created_at: String,
}

pub fn list_characters(conn: &Connection, project_id: &str) -> Result<Vec<CharacterItem>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, role, profile_json, updated_at FROM characters WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(CharacterItem {
            id: r.get(0)?,
            project_id: r.get(1)?,
            name: r.get(2)?,
            role: r.get(3)?,
            profile_json: r.get(4)?,
            updated_at: r.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for x in rows {
        out.push(x?);
    }
    Ok(out)
}

pub fn upsert_character(
    conn: &Connection,
    project_id: &str,
    id: Option<String>,
    name: String,
    role: Option<String>,
    profile_json: Option<String>,
) -> Result<CharacterItem, DbError> {
    let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO characters (id, project_id, name, aliases_json, role, profile_json, created_at, updated_at)
        VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?6)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            role = excluded.role,
            profile_json = excluded.profile_json,
            updated_at = excluded.updated_at
        "#,
        params![id, project_id, name, role, profile_json, now],
    )?;
    Ok(CharacterItem {
        id,
        project_id: project_id.to_string(),
        name,
        role,
        profile_json,
        updated_at: now,
    })
}

pub fn list_locations(conn: &Connection, project_id: &str) -> Result<Vec<LocationItem>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, description, updated_at FROM locations WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(LocationItem {
            id: r.get(0)?,
            project_id: r.get(1)?,
            name: r.get(2)?,
            description: r.get(3)?,
            updated_at: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for x in rows {
        out.push(x?);
    }
    Ok(out)
}

pub fn upsert_location(
    conn: &Connection,
    project_id: &str,
    id: Option<String>,
    name: String,
    description: Option<String>,
) -> Result<LocationItem, DbError> {
    let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO locations (id, project_id, name, description, attributes_json, updated_at)
        VALUES (?1, ?2, ?3, ?4, NULL, ?5)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            updated_at = excluded.updated_at
        "#,
        params![id, project_id, name, description, now],
    )?;
    Ok(LocationItem {
        id,
        project_id: project_id.to_string(),
        name,
        description,
        updated_at: now,
    })
}

pub fn list_worldbuilding(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<WorldbuildingItem>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, category, title, content, updated_at FROM worldbuilding WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(WorldbuildingItem {
            id: r.get(0)?,
            project_id: r.get(1)?,
            category: r.get(2)?,
            title: r.get(3)?,
            content: r.get(4)?,
            updated_at: r.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for x in rows {
        out.push(x?);
    }
    Ok(out)
}

pub fn upsert_worldbuilding(
    conn: &Connection,
    project_id: &str,
    id: Option<String>,
    category: String,
    title: String,
    content: String,
) -> Result<WorldbuildingItem, DbError> {
    let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO worldbuilding (id, project_id, category, title, content, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(id) DO UPDATE SET
            category = excluded.category,
            title = excluded.title,
            content = excluded.content,
            updated_at = excluded.updated_at
        "#,
        params![id, project_id, category, title, content, now],
    )?;
    Ok(WorldbuildingItem {
        id,
        project_id: project_id.to_string(),
        category,
        title,
        content,
        updated_at: now,
    })
}

pub fn list_timeline_events(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<TimelineEventItem>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, document_id, sequence_index, in_story_time, title, summary FROM timeline_events WHERE project_id = ?1 ORDER BY sequence_index ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(TimelineEventItem {
            id: r.get(0)?,
            project_id: r.get(1)?,
            document_id: r.get(2)?,
            sequence_index: r.get(3)?,
            in_story_time: r.get(4)?,
            title: r.get(5)?,
            summary: r.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for x in rows {
        out.push(x?);
    }
    Ok(out)
}

pub fn upsert_timeline_event(
    conn: &Connection,
    project_id: &str,
    id: Option<String>,
    document_id: Option<String>,
    sequence_index: i64,
    in_story_time: Option<String>,
    title: String,
    summary: Option<String>,
) -> Result<TimelineEventItem, DbError> {
    let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    conn.execute(
        r#"
        INSERT INTO timeline_events (id, project_id, document_id, sequence_index, in_story_time, title, summary)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(id) DO UPDATE SET
            document_id = excluded.document_id,
            sequence_index = excluded.sequence_index,
            in_story_time = excluded.in_story_time,
            title = excluded.title,
            summary = excluded.summary
        "#,
        params![
            id,
            project_id,
            document_id,
            sequence_index,
            in_story_time,
            title,
            summary
        ],
    )?;
    Ok(TimelineEventItem {
        id,
        project_id: project_id.to_string(),
        document_id,
        sequence_index,
        in_story_time,
        title,
        summary,
    })
}

pub fn list_facts(conn: &Connection, project_id: &str) -> Result<Vec<FactItem>, DbError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, project_id, subject_type, subject_id, fact_text, certainty,
               evidence_doc_id, evidence_block_id, evidence_quote, created_at
        FROM facts
        WHERE project_id = ?1
        ORDER BY created_at DESC
        "#,
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(FactItem {
            id: r.get(0)?,
            project_id: r.get(1)?,
            subject_type: r.get(2)?,
            subject_id: r.get(3)?,
            fact_text: r.get(4)?,
            certainty: r.get(5)?,
            evidence_doc_id: r.get(6)?,
            evidence_block_id: r.get(7)?,
            evidence_quote: r.get(8)?,
            created_at: r.get(9)?,
        })
    })?;
    let mut out = Vec::new();
    for x in rows {
        out.push(x?);
    }
    Ok(out)
}

pub fn add_fact(
    conn: &Connection,
    project_id: &str,
    subject_type: String,
    subject_id: String,
    fact_text: String,
    certainty: String,
    evidence_doc_id: Option<String>,
    evidence_block_id: Option<String>,
    evidence_quote: Option<String>,
) -> Result<FactItem, DbError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO facts (
            id, project_id, subject_type, subject_id, fact_text, certainty,
            evidence_doc_id, evidence_block_id, evidence_quote, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            id,
            project_id,
            subject_type,
            subject_id,
            fact_text,
            certainty,
            evidence_doc_id,
            evidence_block_id,
            evidence_quote,
            now
        ],
    )?;
    Ok(FactItem {
        id,
        project_id: project_id.to_string(),
        subject_type,
        subject_id,
        fact_text,
        certainty,
        evidence_doc_id,
        evidence_block_id,
        evidence_quote,
        created_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_project_db;

    #[test]
    fn test_worldbuilding_and_grounded_facts() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_project_db(&conn).unwrap();

        let ch = upsert_character(
            &conn,
            "p1",
            None,
            "한서진".into(),
            Some("주인공".into()),
            Some("{\"age\":24}".into()),
        )
        .unwrap();
        assert_eq!(list_characters(&conn, "p1").unwrap().len(), 1);

        let fact = add_fact(
            &conn,
            "p1",
            "character".into(),
            ch.id.clone(),
            "관측소에서 첫 신호를 수신함".into(),
            "confirmed".into(),
            None,
            Some("blk-1".into()),
            Some("그는 첫 신호를 들었다.".into()),
        )
        .unwrap();
        assert_eq!(fact.certainty, "confirmed");
        assert_eq!(list_facts(&conn, "p1").unwrap().len(), 1);
    }
}
