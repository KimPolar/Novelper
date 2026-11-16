use rusqlite::Connection;

pub fn migrate_app_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS app_projects_index (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT 'ko',
            genre TEXT,
            synopsis TEXT,
            db_path TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            protocol TEXT NOT NULL,
            base_url TEXT,
            secret_ref TEXT,
            extra_headers_json TEXT,
            timeout_ms INTEGER NOT NULL DEFAULT 60000,
            max_retries INTEGER NOT NULL DEFAULT 2,
            is_local INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_models (
            id TEXT PRIMARY KEY,
            provider_id TEXT NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
            model_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            capabilities_json TEXT NOT NULL,
            default_options_json TEXT,
            is_custom INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_task_bindings (
            task_type TEXT PRIMARY KEY,
            primary_model_id TEXT REFERENCES ai_models(id) ON DELETE SET NULL,
            fallback_model_ids_json TEXT,
            allow_external_fallback INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
}

pub fn migrate_project_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT 'ko',
            genre TEXT,
            synopsis TEXT,
            style_guide_json TEXT,
            daily_goal_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            parent_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
            doc_type TEXT NOT NULL,
            title TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'draft',
            pov_character_id TEXT,
            location_id TEXT,
            timeline_label TEXT,
            content_json TEXT NOT NULL DEFAULT '{"type":"doc","content":[]}',
            plain_text TEXT NOT NULL DEFAULT '',
            revision INTEGER NOT NULL DEFAULT 1,
            chars_with_space INTEGER NOT NULL DEFAULT 0,
            chars_no_space INTEGER NOT NULL DEFAULT 0,
            word_count INTEGER NOT NULL DEFAULT 0,
            is_trashed INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            doc_id UNINDEXED,
            title,
            plain_text,
            tokenize = 'trigram'
        );

        CREATE TABLE IF NOT EXISTS document_versions (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            revision INTEGER NOT NULL,
            snapshot_type TEXT NOT NULL,
            content_json TEXT NOT NULL,
            plain_text TEXT NOT NULL,
            note TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            aliases_json TEXT,
            role TEXT,
            profile_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS locations (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            attributes_json TEXT,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS worldbuilding (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            category TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS relationships (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS timeline_events (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            document_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
            sequence_index INTEGER NOT NULL,
            in_story_time TEXT,
            title TEXT NOT NULL,
            summary TEXT
        );

        CREATE TABLE IF NOT EXISTS facts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            subject_type TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            fact_text TEXT NOT NULL,
            certainty TEXT NOT NULL,
            evidence_doc_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
            evidence_block_id TEXT,
            evidence_quote TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_conversations (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            document_id TEXT,
            title TEXT NOT NULL,
            messages_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_requests (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            task_type TEXT NOT NULL,
            input_tokens INTEGER,
            output_tokens INTEGER,
            estimated_cost_usd REAL,
            latency_ms INTEGER,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_proposals (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            base_revision INTEGER NOT NULL,
            target_block_id TEXT NOT NULL,
            original_text TEXT NOT NULL,
            proposed_text TEXT NOT NULL,
            rationale TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS embeddings (
            id TEXT PRIMARY KEY,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            block_id TEXT NOT NULL,
            chunk_text TEXT NOT NULL,
            model_id TEXT NOT NULL,
            vector_blob BLOB NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS writing_sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            date_ymd TEXT NOT NULL,
            net_chars_delta INTEGER NOT NULL DEFAULT 0,
            net_words_delta INTEGER NOT NULL DEFAULT 0,
            duration_sec INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
}
