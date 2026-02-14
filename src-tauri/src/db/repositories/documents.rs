use std::path::Path;

use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Row, SqlitePool};

use crate::{
    core::{
        errors::{AppError, AppResult},
        types::{DocNodeDetail, DocNodeSummary, DocumentSummary, GraphNodePosition, NodeType},
    },
    sidecar::types::SidecarNode,
};

fn parse_timestamp(value: String) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|v| v.with_timezone(&Utc))
        .map_err(|err| AppError::Database(format!("invalid timestamp {value}: {err}")))
}

pub async fn find_by_checksum(
    pool: &SqlitePool,
    project_id: &str,
    checksum: &str,
) -> AppResult<Option<DocumentSummary>> {
    let maybe_row = sqlx::query(
        "SELECT id, project_id, name, mime, checksum, pages, created_at FROM documents WHERE project_id = ?1 AND checksum = ?2",
    )
    .bind(project_id)
    .bind(checksum)
    .fetch_optional(pool)
    .await?;

    maybe_row
        .map(map_document_summary)
        .transpose()
}

pub async fn insert_document(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    name: &str,
    mime: &str,
    checksum: &str,
    pages: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO documents (id, project_id, name, mime, checksum, pages)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(name)
    .bind(mime)
    .bind(checksum)
    .bind(pages)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_documents(pool: &SqlitePool, project_id: &str) -> AppResult<Vec<DocumentSummary>> {
    let rows = sqlx::query(
        "SELECT id, project_id, name, mime, checksum, pages, created_at FROM documents WHERE project_id = ?1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_document_summary).collect()
}

pub async fn get_document(pool: &SqlitePool, document_id: &str) -> AppResult<DocumentSummary> {
    let row = sqlx::query(
        "SELECT id, project_id, name, mime, checksum, pages, created_at FROM documents WHERE id = ?1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("document {document_id}")))?;

    map_document_summary(row)
}

pub async fn insert_nodes(
    pool: &SqlitePool,
    document_id: &str,
    nodes: &[SidecarNode],
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    for node in nodes {
        sqlx::query(
            r#"
            INSERT INTO doc_nodes (
              id, document_id, parent_id, node_type, title, text, page_start, page_end,
              bbox_json, metadata_json, ordinal_path
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&node.id)
        .bind(document_id)
        .bind(&node.parent_id)
        .bind(node.node_type.as_str())
        .bind(&node.title)
        .bind(&node.text)
        .bind(node.page_start)
        .bind(node.page_end)
        .bind(node.bbox.to_string())
        .bind(node.metadata.to_string())
        .bind(&node.ordinal_path)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn get_tree(
    pool: &SqlitePool,
    document_id: &str,
    parent_id: Option<&str>,
    depth: i64,
) -> AppResult<Vec<DocNodeSummary>> {
    if depth <= 1 {
        let rows = if let Some(parent) = parent_id {
            sqlx::query(
                r#"
                SELECT id, document_id, parent_id, node_type, title, text, ordinal_path, page_start, page_end
                FROM doc_nodes
                WHERE document_id = ?1 AND parent_id = ?2
                ORDER BY ordinal_path
                "#,
            )
            .bind(document_id)
            .bind(parent)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, document_id, parent_id, node_type, title, text, ordinal_path, page_start, page_end
                FROM doc_nodes
                WHERE document_id = ?1 AND parent_id IS NULL
                ORDER BY ordinal_path
                "#,
            )
            .bind(document_id)
            .fetch_all(pool)
            .await?
        };
        return rows.into_iter().map(map_node_summary).collect();
    }

    let roots_query = if parent_id.is_some() {
        "id = ?2"
    } else {
        "parent_id IS NULL"
    };
    let sql = format!(
        r#"
        WITH RECURSIVE tree(id, depth) AS (
          SELECT id, 0
          FROM doc_nodes
          WHERE document_id = ?1 AND {roots_query}
          UNION ALL
          SELECT child.id, tree.depth + 1
          FROM doc_nodes child
          JOIN tree ON child.parent_id = tree.id
          WHERE child.document_id = ?1 AND tree.depth < ?3
        )
        SELECT dn.id, dn.document_id, dn.parent_id, dn.node_type, dn.title, dn.text, dn.ordinal_path, dn.page_start, dn.page_end
        FROM doc_nodes dn
        JOIN tree ON dn.id = tree.id
        ORDER BY CASE WHEN dn.parent_id IS NULL THEN 0 ELSE 1 END, dn.ordinal_path
        "#
    );
    let mut query = sqlx::query(&sql).bind(document_id);
    if let Some(parent) = parent_id {
        query = query.bind(parent);
    } else {
        query = query.bind("");
    }
    let rows = query.bind(depth).fetch_all(pool).await?;
    rows.into_iter().map(map_node_summary).collect()
}

pub async fn get_project_tree(
    pool: &SqlitePool,
    project_id: &str,
    depth: i64,
) -> AppResult<Vec<DocNodeSummary>> {
    if depth <= 1 {
        let rows = sqlx::query(
            r#"
            SELECT dn.id, dn.document_id, dn.parent_id, dn.node_type, dn.title, dn.text, dn.ordinal_path, dn.page_start, dn.page_end
            FROM doc_nodes dn
            JOIN documents d ON d.id = dn.document_id
            WHERE d.project_id = ?1 AND dn.parent_id IS NULL
            ORDER BY d.created_at ASC, dn.ordinal_path
            "#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;
        return rows.into_iter().map(map_node_summary).collect();
    }

    let rows = sqlx::query(
        r#"
        WITH RECURSIVE tree(id, depth) AS (
          SELECT dn.id, 0
          FROM doc_nodes dn
          JOIN documents d ON d.id = dn.document_id
          WHERE d.project_id = ?1 AND dn.parent_id IS NULL
          UNION ALL
          SELECT child.id, tree.depth + 1
          FROM doc_nodes child
          JOIN tree ON child.parent_id = tree.id
          WHERE tree.depth < ?2
        )
        SELECT dn.id, dn.document_id, dn.parent_id, dn.node_type, dn.title, dn.text, dn.ordinal_path, dn.page_start, dn.page_end
        FROM doc_nodes dn
        JOIN tree ON dn.id = tree.id
        ORDER BY dn.document_id, CASE WHEN dn.parent_id IS NULL THEN 0 ELSE 1 END, dn.ordinal_path
        "#,
    )
    .bind(project_id)
    .bind(depth)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(map_node_summary).collect()
}

pub async fn get_node(pool: &SqlitePool, node_id: &str) -> AppResult<DocNodeDetail> {
    let row = sqlx::query(
        r#"
        SELECT id, document_id, parent_id, node_type, title, text, ordinal_path, page_start, page_end, bbox_json, metadata_json
        FROM doc_nodes
        WHERE id = ?1
        "#,
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("node {node_id}")))?;

    map_node_detail(row)
}

pub async fn delete_document(pool: &SqlitePool, document_id: &str) -> AppResult<bool> {
    let changed = sqlx::query("DELETE FROM documents WHERE id = ?1")
        .bind(document_id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(changed > 0)
}

pub async fn get_graph_layout(
    pool: &SqlitePool,
    document_id: &str,
) -> AppResult<Vec<GraphNodePosition>> {
    let rows = sqlx::query(
        r#"
        SELECT node_id, x, y
        FROM graph_layouts
        WHERE document_id = ?1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(GraphNodePosition {
                node_id: row.try_get("node_id")?,
                x: row.try_get("x")?,
                y: row.try_get("y")?,
            })
        })
        .collect()
}

pub async fn save_graph_layout(
    pool: &SqlitePool,
    document_id: &str,
    positions: &[GraphNodePosition],
) -> AppResult<usize> {
    let mut tx = pool.begin().await?;
    let mut saved = 0usize;

    if positions.is_empty() {
        sqlx::query("DELETE FROM graph_layouts WHERE document_id = ?1")
            .bind(document_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(0);
    }

    let mut cleanup = QueryBuilder::new(
        "DELETE FROM graph_layouts WHERE document_id = ",
    );
    cleanup
        .push_bind(document_id)
        .push(" AND node_id NOT IN (");
    let mut separated = cleanup.separated(", ");
    for position in positions {
        separated.push_bind(&position.node_id);
    }
    cleanup.push(")");
    cleanup.build().execute(&mut *tx).await?;

    for position in positions {
        let affected = sqlx::query(
            r#"
            INSERT INTO graph_layouts (document_id, node_id, x, y, updated_at)
            SELECT ?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE EXISTS (
              SELECT 1
              FROM doc_nodes
              WHERE document_id = ?1 AND id = ?2
            )
            ON CONFLICT(document_id, node_id) DO UPDATE SET
              x = excluded.x,
              y = excluded.y,
              updated_at = excluded.updated_at
            "#,
        )
        .bind(document_id)
        .bind(&position.node_id)
        .bind(position.x)
        .bind(position.y)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        saved += affected as usize;
    }

    tx.commit().await?;
    Ok(saved)
}

pub async fn export_markdown(
    pool: &SqlitePool,
    document_id: &str,
    export_path: &Path,
) -> AppResult<()> {
    let document = get_document(pool, document_id).await?;
    let nodes = sqlx::query(
        r#"
        SELECT id, document_id, parent_id, node_type, title, text, ordinal_path, page_start, page_end, bbox_json, metadata_json
        FROM doc_nodes
        WHERE document_id = ?1
        ORDER BY ordinal_path
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    let mut out = String::new();
    out.push_str("# ");
    out.push_str(&document.name);
    out.push_str("\n\n");

    for row in nodes {
        let node = map_node_detail(row)?;
        match node.node_type {
            NodeType::Document => {
                if !node.text.is_empty() {
                    out.push_str(&node.text);
                    out.push_str("\n\n");
                }
            }
            NodeType::Section => {
                out.push_str("## ");
                out.push_str(&node.title);
                out.push('\n');
                if !node.text.is_empty() {
                    out.push_str(&node.text);
                    out.push_str("\n\n");
                }
            }
            NodeType::Subsection => {
                out.push_str("### ");
                out.push_str(&node.title);
                out.push('\n');
                if !node.text.is_empty() {
                    out.push_str(&node.text);
                    out.push_str("\n\n");
                }
            }
            _ => {
                if !node.title.is_empty() {
                    out.push_str("**");
                    out.push_str(&node.title);
                    out.push_str("**\n");
                }
                if !node.text.is_empty() {
                    out.push_str(&node.text);
                    out.push_str("\n\n");
                }
            }
        }
    }

    std::fs::write(export_path, out).map_err(|err| AppError::Io(err.to_string()))?;
    Ok(())
}

fn map_document_summary(row: sqlx::sqlite::SqliteRow) -> AppResult<DocumentSummary> {
    let created_at: String = row.try_get("created_at")?;
    Ok(DocumentSummary {
        id: row.try_get("id")?,
        project_id: row.try_get("project_id")?,
        name: row.try_get("name")?,
        mime: row.try_get("mime")?,
        checksum: row.try_get("checksum")?,
        pages: row.try_get("pages")?,
        created_at: parse_timestamp(created_at)?,
    })
}

fn map_node_summary(row: sqlx::sqlite::SqliteRow) -> AppResult<DocNodeSummary> {
    let node_type: String = row.try_get("node_type")?;
    Ok(DocNodeSummary {
        id: row.try_get("id")?,
        document_id: row.try_get("document_id")?,
        parent_id: row.try_get("parent_id")?,
        node_type: NodeType::from_str(&node_type),
        title: row.try_get("title")?,
        text: row.try_get("text")?,
        ordinal_path: row.try_get("ordinal_path")?,
        page_start: row.try_get("page_start")?,
        page_end: row.try_get("page_end")?,
    })
}

fn map_node_detail(row: sqlx::sqlite::SqliteRow) -> AppResult<DocNodeDetail> {
    let node_type: String = row.try_get("node_type")?;
    let bbox_json: String = row.try_get("bbox_json")?;
    let metadata_json: String = row.try_get("metadata_json")?;
    Ok(DocNodeDetail {
        id: row.try_get("id")?,
        document_id: row.try_get("document_id")?,
        parent_id: row.try_get("parent_id")?,
        node_type: NodeType::from_str(&node_type),
        title: row.try_get("title")?,
        text: row.try_get("text")?,
        ordinal_path: row.try_get("ordinal_path")?,
        page_start: row.try_get("page_start")?,
        page_end: row.try_get("page_end")?,
        bbox_json: serde_json::from_str(&bbox_json).unwrap_or_else(|_| serde_json::json!({})),
        metadata_json: serde_json::from_str(&metadata_json).unwrap_or_else(|_| serde_json::json!({})),
    })
}
