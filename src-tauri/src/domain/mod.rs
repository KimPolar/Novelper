use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub language: String,
    pub genre: Option<String>,
    pub synopsis: Option<String>,
    pub path: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub id: String,
    pub title: String,
    pub language: String,
    pub genre: Option<String>,
    pub synopsis: Option<String>,
    pub style_guide_json: Option<String>,
    pub daily_goal_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTreeItem {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub doc_type: String,
    pub title: String,
    pub sort_order: i64,
    pub status: String,
    pub revision: i64,
    pub chars_with_space: i64,
    pub chars_no_space: i64,
    pub word_count: i64,
    pub is_trashed: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDetail {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub doc_type: String,
    pub title: String,
    pub sort_order: i64,
    pub status: String,
    pub pov_character_id: Option<String>,
    pub location_id: Option<String>,
    pub timeline_label: Option<String>,
    pub content_json: String,
    pub plain_text: String,
    pub revision: i64,
    pub chars_with_space: i64,
    pub chars_no_space: i64,
    pub word_count: i64,
    pub is_trashed: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub title: String,
    pub language: String,
    pub genre: Option<String>,
    pub synopsis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentInput {
    pub parent_id: Option<String>,
    pub doc_type: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentInput {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub content_json: Option<String>,
    pub plain_text: Option<String>,
    pub expected_revision: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderItem {
    pub id: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtsSearchHit {
    pub doc_id: String,
    pub title: String,
    pub snippet: String,
}

/// 한국어(공백 포함/제외) 및 영어 단어 수를 계산합니다.
pub fn compute_text_stats(plain_text: &str) -> (i64, i64, i64) {
    let chars_with_space = plain_text.chars().count() as i64;
    let chars_no_space = plain_text.chars().filter(|c| !c.is_whitespace()).count() as i64;
    let word_count = plain_text.split_whitespace().count() as i64;
    (chars_with_space, chars_no_space, word_count)
}
