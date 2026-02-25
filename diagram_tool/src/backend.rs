#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle};
use crate::models::schema::validate_schema;
use crate::models::validation::validate_document;
use crate::mutation::pipeline::{run_mutation_with_policy, RevisionPolicy};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use redb::{Database, TableDefinition};
#[cfg(feature = "server")]
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedWorkspace {
    pub schema_version: u32,
    pub document: DiagramDocument,
    pub tool_mode: String,
    pub edge_style: EdgeStyle,
    pub arrow_type: ArrowType,
}

impl PersistedWorkspace {
    pub const SCHEMA_VERSION: u32 = 1;
}

fn validate_workspace_document(document: &DiagramDocument) -> Result<(), ServerFnError> {
    validate_schema(document)
        .map_err(|err| ServerFnError::new(format!("schema validation error: {err}")))?;

    validate_document(document).first().map_or_else(
        || Ok(()),
        |issue| {
            Err(ServerFnError::new(format!(
                "semantic validation error [{}]: {}",
                issue.code, issue.message
            )))
        },
    )
}

#[cfg(feature = "server")]
const DIAGRAM_TABLE: TableDefinition<&str, &str> = TableDefinition::new("diagrams");

#[cfg(feature = "server")]
fn database_path() -> PathBuf {
    std::env::var("DIAGRAM_TOOL_DB")
        .map_or_else(|_| PathBuf::from("diagram_tool.redb"), PathBuf::from)
}

#[cfg(feature = "server")]
fn with_database<T>(f: impl FnOnce(&Database) -> Result<T, ServerFnError>) -> Result<T, ServerFnError> {
    let db = Database::create(database_path())
        .map_err(|err| ServerFnError::new(format!("database open error: {err}")))?;

    let write_txn = db
        .begin_write()
        .map_err(|err| ServerFnError::new(format!("database write transaction error: {err}")))?;
    let _ = write_txn
        .open_table(DIAGRAM_TABLE)
        .map_err(|err| ServerFnError::new(format!("database table open error: {err}")))?;
    write_txn
        .commit()
        .map_err(|err| ServerFnError::new(format!("database commit error: {err}")))?;

    f(&db)
}

#[server]
pub async fn backend_health() -> Result<String, ServerFnError> {
    std::future::ready(()).await;

    #[cfg(feature = "server")]
    {
        with_database(|_| Ok(String::from("Connected to Dioxus backend")))
    }

    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new(
            "Backend health is only available in server mode",
        ))
    }
}

#[server]
pub async fn save_workspace_to_backend(workspace: PersistedWorkspace) -> Result<String, ServerFnError> {
    std::future::ready(()).await;

    validate_workspace_document(&workspace.document)?;

    let node_count = workspace.document.document.nodes.len();
    let edge_count = workspace.document.document.edges.len();
    let serialized = serde_json::to_string(&workspace)
        .map_err(|err| ServerFnError::new(format!("serialize error: {err}")))?;

    #[cfg(feature = "server")]
    {
        with_database(move |db| {
            let write_txn = db.begin_write().map_err(|err| {
                ServerFnError::new(format!("database write transaction error: {err}"))
            })?;

            {
                let mut table = write_txn.open_table(DIAGRAM_TABLE).map_err(|err| {
                    ServerFnError::new(format!("database table open error: {err}"))
                })?;
                table
                    .insert("default", serialized.as_str())
                    .map_err(|err| ServerFnError::new(format!("database insert error: {err}")))?;
            }

            write_txn
                .commit()
                .map_err(|err| ServerFnError::new(format!("database commit error: {err}")))?;

            Ok(format!(
                "Saved {node_count} nodes and {edge_count} edges to backend"
            ))
        })
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = serialized;
        let _ = node_count;
        let _ = edge_count;
        Err(ServerFnError::new(
            "Backend save is only available in server mode",
        ))
    }
}

#[server]
pub async fn load_workspace_from_backend() -> Result<PersistedWorkspace, ServerFnError> {
    std::future::ready(()).await;

    #[cfg(feature = "server")]
    {
        with_database(|db| {
            let read_txn = db.begin_read().map_err(|err| {
                ServerFnError::new(format!("database read transaction error: {err}"))
            })?;
            let table = read_txn.open_table(DIAGRAM_TABLE).map_err(|err| {
                ServerFnError::new(format!("database table open error: {err}"))
            })?;

            let value = table
                .get("default")
                .map_err(|err| ServerFnError::new(format!("database read error: {err}")))?;

            value.map_or_else(
                || {
                    Ok(PersistedWorkspace {
                        schema_version: PersistedWorkspace::SCHEMA_VERSION,
                        document: DiagramDocument::default(),
                        tool_mode: String::from("select"),
                        edge_style: EdgeStyle::default(),
                        arrow_type: ArrowType::default(),
                    })
                },
                |entry| {
                    let raw = entry.value();
                    serde_json::from_str::<PersistedWorkspace>(raw)
                        .map_err(|err| ServerFnError::new(format!("deserialize error: {err}")))
                        .and_then(|workspace| {
                            if workspace.schema_version == PersistedWorkspace::SCHEMA_VERSION {
                                validate_workspace_document(&workspace.document)
                                    .map(|()| workspace)
                            } else {
                                Err(ServerFnError::new(format!(
                                    "schema version mismatch: expected {}, got {}",
                                    PersistedWorkspace::SCHEMA_VERSION,
                                    workspace.schema_version
                                )))
                            }
                        })
                },
            )
        })
    }

    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new(
            "Backend load is only available in server mode",
        ))
    }
}

#[server]
pub async fn ingest_document_json_to_backend(raw_document: String) -> Result<String, ServerFnError> {
    std::future::ready(()).await;

    let incoming = serde_json::from_str::<DiagramDocument>(&raw_document)
        .map_err(|err| ServerFnError::new(format!("ingest parse error: {err}")))?;

    let current = load_workspace_from_backend().await?;
    let next_doc = run_mutation_with_policy(
        &current.document,
        RevisionPolicy::Preserve,
        |_| Ok(incoming),
    )
    .map_err(|err| ServerFnError::new(format!("ingest validation error: {err}")))?;

    let next_workspace = PersistedWorkspace {
        schema_version: PersistedWorkspace::SCHEMA_VERSION,
        document: next_doc,
        tool_mode: current.tool_mode,
        edge_style: current.edge_style,
        arrow_type: current.arrow_type,
    };

    save_workspace_to_backend(next_workspace).await
}
