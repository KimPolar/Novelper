use crate::db::project_repo::{get_document, get_project_detail, search_documents_fts};
use crate::db::world_repo::{
    list_characters, list_facts, list_locations, list_timeline_events, list_worldbuilding,
};
use crate::db::DbError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltContext {
    pub context_text: String,
    pub estimated_tokens: usize,
    pub included_sections: Vec<String>,
}

pub fn estimate_tokens(text: &str) -> usize {
    // 한국어/영어 혼합 기준 보수적 토큰 추정
    (text.chars().count() as f64 * 0.65).ceil() as usize
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

pub fn build_scene_context(
    proj_conn: &Connection,
    project_id: &str,
    doc_id: Option<&str>,
    query: Option<&str>,
    token_budget: usize,
) -> Result<BuiltContext, DbError> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut included_sections = Vec::new();

    if let Ok(proj) = get_project_detail(proj_conn) {
        let mut overview = format!("Title: {}\nLanguage: {}", proj.title, proj.language);
        if let Some(g) = proj.genre {
            overview.push_str(&format!("\nGenre: {}", g));
        }
        if let Some(s) = proj.synopsis {
            overview.push_str(&format!("\nSynopsis: {}", s));
        }
        sections.push(("Project Overview".to_string(), overview));
    }

    if let Some(did) = doc_id {
        if let Ok(doc) = get_document(proj_conn, did) {
            sections.push((
                format!("Current Scene ({})", doc.title),
                doc.plain_text,
            ));
        }
    }

    let chars = list_characters(proj_conn, project_id)?;
    if !chars.is_empty() {
        let body = chars
            .iter()
            .map(|c| {
                format!(
                    "- {} ({}): {}",
                    c.name,
                    c.role.as_deref().unwrap_or(""),
                    c.profile_json.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(("Characters".to_string(), body));
    }

    let locs = list_locations(proj_conn, project_id)?;
    if !locs.is_empty() {
        let body = locs
            .iter()
            .map(|l| format!("- {}: {}", l.name, l.description.as_deref().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(("Locations".to_string(), body));
    }

    let world = list_worldbuilding(proj_conn, project_id)?;
    if !world.is_empty() {
        let body = world
            .iter()
            .map(|w| format!("- [{}] {}: {}", w.category, w.title, w.content))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(("Worldbuilding".to_string(), body));
    }

    let timeline = list_timeline_events(proj_conn, project_id)?;
    if !timeline.is_empty() {
        let body = timeline
            .iter()
            .map(|e| {
                format!(
                    "{}. [{}] {}: {}",
                    e.sequence_index,
                    e.in_story_time.as_deref().unwrap_or(""),
                    e.title,
                    e.summary.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(("Timeline".to_string(), body));
    }

    let facts = list_facts(proj_conn, project_id)?;
    if !facts.is_empty() {
        let body = facts
            .iter()
            .map(|f| format!("- [{}] {}", f.certainty, f.fact_text))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(("Grounded Facts".to_string(), body));
    }

    if let Some(q) = query {
        if !q.trim().is_empty() {
            if let Ok(hits) = search_documents_fts(proj_conn, q) {
                if !hits.is_empty() {
                    let body = hits
                        .into_iter()
                        .take(5)
                        .map(|h| format!("- {}: {}", h.title, h.snippet))
                        .collect::<Vec<_>>()
                        .join("\n");
                    sections.push(("Related Manuscript Snippets".to_string(), body));
                }
            }
        }
    }

    let mut context_text = String::new();
    let mut used_tokens = 0usize;

    for (title, body) in sections {
        let block = format!("[{}]\n{}\n\n", title, body);
        let cost = estimate_tokens(&block);
        if used_tokens + cost <= token_budget {
            context_text.push_str(&block);
            used_tokens += cost;
            included_sections.push(title);
        }
    }

    Ok(BuiltContext {
        context_text,
        estimated_tokens: used_tokens,
        included_sections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_project_db;
    use crate::db::project_repo::create_project;
    use crate::db::world_repo::upsert_character;
    use crate::domain::CreateProjectInput;

    #[test]
    fn test_context_builder_respects_token_budget() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_project_db(&conn).unwrap();

        let proj = create_project(
            &conn,
            CreateProjectInput {
                title: "은하 관측소".into(),
                language: "ko".into(),
                genre: Some("SF".into()),
                synopsis: Some("우주 정거장의 미스터리".into()),
            },
        )
        .unwrap();

        upsert_character(&conn, &proj.id, None, "이도현".into(), Some("대원".into()), None).unwrap();

        let ctx = build_scene_context(&conn, &proj.id, None, None, 2000).unwrap();
        assert!(ctx.included_sections.contains(&"Project Overview".to_string()));
        assert!(ctx.included_sections.contains(&"Characters".to_string()));
        assert!(ctx.estimated_tokens <= 2000);

        let sim = cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]);
        assert!((sim - 1.0).abs() < 1e-5);
    }
}
