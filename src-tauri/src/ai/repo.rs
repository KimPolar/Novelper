use crate::ai::{AIModelInfo, AIProviderConfig, ModelCapabilities, ProtocolKind, TaskBinding};
use crate::db::DbError;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

pub fn list_providers(app_conn: &Connection) -> Result<Vec<AIProviderConfig>, DbError> {
    let mut stmt = app_conn.prepare(
        r#"
        SELECT id, name, protocol, base_url, secret_ref, extra_headers_json,
               timeout_ms, max_retries, is_local, enabled
        FROM ai_providers
        ORDER BY created_at ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let proto_str: String = row.get(2)?;
        let headers_json: Option<String> = row.get(5)?;
        let is_local: i64 = row.get(8)?;
        let enabled: i64 = row.get(9)?;
        Ok(AIProviderConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            protocol: ProtocolKind::from_str(&proto_str).unwrap_or(ProtocolKind::OpenaiCompat),
            base_url: row.get(3)?,
            secret_ref: row.get(4)?,
            extra_headers: headers_json.and_then(|s| serde_json::from_str(&s).ok()),
            timeout_ms: row.get::<_, i64>(6)? as u64,
            max_retries: row.get::<_, i64>(7)? as u32,
            is_local: is_local != 0,
            enabled: enabled != 0,
        })
    })?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub fn upsert_provider(
    app_conn: &Connection,
    cfg: &AIProviderConfig,
) -> Result<AIProviderConfig, DbError> {
    let now = Utc::now().to_rfc3339();
    let headers_json = cfg
        .extra_headers
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    app_conn.execute(
        r#"
        INSERT INTO ai_providers (
            id, name, protocol, base_url, secret_ref, extra_headers_json,
            timeout_ms, max_retries, is_local, enabled, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            protocol = excluded.protocol,
            base_url = excluded.base_url,
            secret_ref = excluded.secret_ref,
            extra_headers_json = excluded.extra_headers_json,
            timeout_ms = excluded.timeout_ms,
            max_retries = excluded.max_retries,
            is_local = excluded.is_local,
            enabled = excluded.enabled,
            updated_at = excluded.updated_at
        "#,
        params![
            cfg.id,
            cfg.name,
            cfg.protocol.as_str(),
            cfg.base_url,
            cfg.secret_ref,
            headers_json,
            cfg.timeout_ms as i64,
            cfg.max_retries as i64,
            if cfg.is_local { 1 } else { 0 },
            if cfg.enabled { 1 } else { 0 },
            now
        ],
    )?;
    Ok(cfg.clone())
}

pub fn delete_provider(app_conn: &Connection, provider_id: &str) -> Result<(), DbError> {
    app_conn.execute("DELETE FROM ai_providers WHERE id = ?1", params![provider_id])?;
    Ok(())
}

pub fn list_models(
    app_conn: &Connection,
    provider_id: Option<&str>,
) -> Result<Vec<AIModelInfo>, DbError> {
    let sql = if provider_id.is_some() {
        "SELECT id, provider_id, model_id, display_name, capabilities_json, default_options_json, is_custom FROM ai_models WHERE provider_id = ?1 ORDER BY created_at ASC"
    } else {
        "SELECT id, provider_id, model_id, display_name, capabilities_json, default_options_json, is_custom FROM ai_models ORDER BY created_at ASC"
    };
    let mut stmt = app_conn.prepare(sql)?;
    let map_row = |row: &rusqlite::Row| {
        let caps_str: String = row.get(4)?;
        let opts_str: Option<String> = row.get(5)?;
        let is_custom: i64 = row.get(6)?;
        Ok(AIModelInfo {
            id: row.get(0)?,
            provider_id: row.get(1)?,
            model_id: row.get(2)?,
            display_name: row.get(3)?,
            capabilities: serde_json::from_str(&caps_str).unwrap_or_default(),
            default_options: opts_str.and_then(|s| serde_json::from_str(&s).ok()),
            is_custom: is_custom != 0,
        })
    };
    let mut result = Vec::new();
    if let Some(pid) = provider_id {
        let rows = stmt.query_map(params![pid], map_row)?;
        for item in rows {
            result.push(item?);
        }
    } else {
        let rows = stmt.query_map([], map_row)?;
        for item in rows {
            result.push(item?);
        }
    }
    Ok(result)
}

pub fn upsert_model(app_conn: &Connection, model: &AIModelInfo) -> Result<AIModelInfo, DbError> {
    let now = Utc::now().to_rfc3339();
    let caps_json = serde_json::to_string(&model.capabilities).unwrap_or_default();
    let opts_json = model
        .default_options
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    app_conn.execute(
        r#"
        INSERT INTO ai_models (id, provider_id, model_id, display_name, capabilities_json, default_options_json, is_custom, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            display_name = excluded.display_name,
            capabilities_json = excluded.capabilities_json,
            default_options_json = excluded.default_options_json,
            is_custom = excluded.is_custom
        "#,
        params![
            model.id,
            model.provider_id,
            model.model_id,
            model.display_name,
            caps_json,
            opts_json,
            if model.is_custom { 1 } else { 0 },
            now
        ],
    )?;
    Ok(model.clone())
}

pub fn delete_model(app_conn: &Connection, model_id: &str) -> Result<(), DbError> {
    app_conn.execute("DELETE FROM ai_models WHERE id = ?1", params![model_id])?;
    Ok(())
}

pub fn list_task_bindings(app_conn: &Connection) -> Result<Vec<TaskBinding>, DbError> {
    let mut stmt = app_conn.prepare(
        "SELECT task_type, primary_model_id, fallback_model_ids_json, allow_external_fallback FROM ai_task_bindings",
    )?;
    let rows = stmt.query_map([], |row| {
        let fb_json: Option<String> = row.get(2)?;
        let allow_ext: i64 = row.get(3)?;
        Ok(TaskBinding {
            task_type: row.get(0)?,
            primary_model_id: row.get(1)?,
            fallback_model_ids: fb_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            allow_external_fallback: allow_ext != 0,
        })
    })?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub fn upsert_task_binding(app_conn: &Connection, binding: &TaskBinding) -> Result<(), DbError> {
    let fb_json = serde_json::to_string(&binding.fallback_model_ids).unwrap_or_default();
    app_conn.execute(
        r#"
        INSERT INTO ai_task_bindings (task_type, primary_model_id, fallback_model_ids_json, allow_external_fallback)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(task_type) DO UPDATE SET
            primary_model_id = excluded.primary_model_id,
            fallback_model_ids_json = excluded.fallback_model_ids_json,
            allow_external_fallback = excluded.allow_external_fallback
        "#,
        params![
            binding.task_type,
            binding.primary_model_id,
            fb_json,
            if binding.allow_external_fallback { 1 } else { 0 }
        ],
    )?;
    Ok(())
}

pub fn find_provider_and_model(
    app_conn: &Connection,
    model_record_id: &str,
) -> Result<Option<(AIProviderConfig, AIModelInfo)>, DbError> {
    let model: Option<AIModelInfo> = app_conn
        .query_row(
            "SELECT id, provider_id, model_id, display_name, capabilities_json, default_options_json, is_custom FROM ai_models WHERE id = ?1",
            params![model_record_id],
            |row| {
                let caps_str: String = row.get(4)?;
                let opts_str: Option<String> = row.get(5)?;
                let is_custom: i64 = row.get(6)?;
                Ok(AIModelInfo {
                    id: row.get(0)?,
                    provider_id: row.get(1)?,
                    model_id: row.get(2)?,
                    display_name: row.get(3)?,
                    capabilities: serde_json::from_str::<ModelCapabilities>(&caps_str).unwrap_or_default(),
                    default_options: opts_str.and_then(|s| serde_json::from_str(&s).ok()),
                    is_custom: is_custom != 0,
                })
            },
        )
        .optional()?;

    let Some(m) = model else { return Ok(None) };
    let providers = list_providers(app_conn)?;
    let provider = providers.into_iter().find(|p| p.id == m.provider_id);
    Ok(provider.map(|p| (p, m)))
}

pub fn log_ai_request(
    proj_conn: &Connection,
    project_id: &str,
    provider_id: &str,
    model_id: &str,
    task_type: &str,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    latency_ms: i64,
    status: &str,
) -> Result<(), DbError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    proj_conn.execute(
        r#"
        INSERT INTO ai_requests (id, project_id, provider_id, model_id, task_type, input_tokens, output_tokens, estimated_cost_usd, latency_ms, status, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9, ?10)
        "#,
        params![
            id,
            project_id,
            provider_id,
            model_id,
            task_type,
            input_tokens.map(|v| v as i64),
            output_tokens.map(|v| v as i64),
            latency_ms,
            status,
            now
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::migrate_app_db;

    #[test]
    fn test_ai_provider_model_and_binding_crud() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_app_db(&conn).unwrap();

        let provider = AIProviderConfig {
            id: "prov-1".to_string(),
            name: "Local LM Studio".to_string(),
            protocol: ProtocolKind::OpenaiCompat,
            base_url: Some("http://localhost:1234/v1".to_string()),
            secret_ref: None,
            extra_headers: None,
            timeout_ms: 30_000,
            max_retries: 1,
            is_local: true,
            enabled: true,
        };
        upsert_provider(&conn, &provider).unwrap();
        assert_eq!(list_providers(&conn).unwrap().len(), 1);

        let model = AIModelInfo {
            id: "mod-1".to_string(),
            provider_id: "prov-1".to_string(),
            model_id: "qwen-local".to_string(),
            display_name: "Qwen Local".to_string(),
            capabilities: ModelCapabilities::default(),
            default_options: None,
            is_custom: true,
        };
        upsert_model(&conn, &model).unwrap();
        assert_eq!(list_models(&conn, Some("prov-1")).unwrap().len(), 1);

        upsert_task_binding(
            &conn,
            &TaskBinding {
                task_type: "draft".to_string(),
                primary_model_id: Some("mod-1".to_string()),
                fallback_model_ids: vec![],
                allow_external_fallback: false,
            },
        )
        .unwrap();
        let bindings = list_task_bindings(&conn).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].primary_model_id.as_deref(), Some("mod-1"));
    }
}
