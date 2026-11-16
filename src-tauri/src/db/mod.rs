pub mod migrations;
pub mod project_repo;
pub mod proposal_repo;
pub mod version_repo;
pub mod world_repo;

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No active project is open")]
    NoActiveProject,
    #[error("Document not found: {0}")]
    NotFound(String),
    #[error("Revision conflict: expected {expected}, actual {actual}")]
    RevisionConflict { expected: i64, actual: i64 },
}

pub struct ActiveProject {
    pub project_id: String,
    pub db_path: PathBuf,
    pub conn: Connection,
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub app_db: Mutex<Connection>,
    pub active_project: Mutex<Option<ActiveProject>>,
}

impl AppState {
    pub fn init(data_dir: PathBuf) -> Result<Self, DbError> {
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(data_dir.join("projects"))?;
        let app_db_path = data_dir.join("app.db");
        let app_conn = Connection::open(app_db_path)?;
        migrations::migrate_app_db(&app_conn)?;

        Ok(Self {
            data_dir,
            app_db: Mutex::new(app_conn),
            active_project: Mutex::new(None),
        })
    }

    pub fn projects_dir(&self) -> PathBuf {
        self.data_dir.join("projects")
    }
}

pub fn open_project_connection(db_path: &Path) -> Result<Connection, DbError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path)?;
    migrations::migrate_project_db(&conn)?;
    Ok(conn)
}
