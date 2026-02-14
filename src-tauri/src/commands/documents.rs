use std::path::PathBuf;

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::{
    core::{
        errors::{AppError, AppResult},
        types::{
            DeleteDocumentResponse, DocumentPreviewBlock, ExportMarkdownResponse, GetDocumentPreviewResponse,
            GetGraphLayoutResponse, GetNodeResponse, GetTreeResponse, GraphNodePosition,
            IngestDocumentResponse, IngestProgressEvent, ListDocumentsResponse, OpenDocumentResponse,
            SaveGraphLayoutResponse,
        },
    },
    db::repositories::documents,
    sidecar::native_parser,
    AppState,
};

fn checksum_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
pub async fn ingest_document(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    file_path: String,
    mime_type: String,
    display_name: Option<String>,
) -> AppResult<IngestDocumentResponse> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(AppError::NotFound(format!("file {file_path}")));
    }

    let bytes = std::fs::read(&path).map_err(|err| AppError::Io(err.to_string()))?;
    let checksum = checksum_bytes(&bytes);
    
    // Check for existing document with same checksum
    if let Some(existing) = documents::find_by_checksum(state.db.pool(), &project_id, &checksum).await? {
        // Try to get the tree for the existing document
        match documents::get_tree(state.db.pool(), &existing.id, None, 8).await {
            Ok(existing_nodes) => {
                // Verify the document has a valid root node
                if let Some(root) = existing_nodes.iter().find(|node| node.parent_id.is_none()) {
                    let section_count = existing_nodes
                        .iter()
                        .filter(|node| {
                            matches!(
                                node.node_type,
                                crate::core::types::NodeType::Section | crate::core::types::NodeType::Subsection
                            )
                        })
                        .count();
                    
                    eprintln!("Document already exists with checksum {}, returning cached result", checksum);
                    return Ok(IngestDocumentResponse {
                        document_id: existing.id,
                        root_node_id: root.id.clone(),
                        node_count: existing_nodes.len(),
                        section_count,
                    });
                } else {
                    // Document exists but has no root node - it's corrupted, delete it
                    eprintln!("Found corrupted document {} (no root node), deleting and re-parsing", existing.id);
                    let _ = documents::delete_document(state.db.pool(), &existing.id).await;
                }
            }
            Err(e) => {
                // Failed to get tree - document is corrupted, delete it
                eprintln!("Found corrupted document {} (failed to get tree: {}), deleting and re-parsing", existing.id, e);
                let _ = documents::delete_document(state.db.pool(), &existing.id).await;
            }
        }
    }

    let job_id = Uuid::new_v4().to_string();
    let _ = app.emit(
        "ingest/progress",
        IngestProgressEvent {
            job_id: job_id.clone(),
            stage: "queued".to_string(),
            percent: 0,
            message: "Starting ingestion".to_string(),
        },
    );

    let _ = app.emit(
        "ingest/progress",
        IngestProgressEvent {
            job_id: job_id.clone(),
            stage: "parse".to_string(),
            percent: 30,
            message: "Parsing document\u{2026}".to_string(),
        },
    );
    
    let parsed = match native_parser::parse(&path, &mime_type) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Document parsing failed for {:?}: {:?}", path, e);
            return Err(e);
        }
    };

    let document_id = Uuid::new_v4().to_string();
    let name = display_name.unwrap_or_else(|| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| parsed.document.title.clone())
    });

    documents::insert_document(
        state.db.pool(),
        &document_id,
        &project_id,
        &name,
        &mime_type,
        &checksum,
        parsed.document.pages,
    )
    .await?;

    if let Err(err) = documents::insert_nodes(state.db.pool(), &document_id, &parsed.nodes).await {
        let _ = documents::delete_document(state.db.pool(), &document_id).await;
        return Err(err);
    }

    let _ = app.emit(
        "ingest/progress",
        IngestProgressEvent {
            job_id,
            stage: "finalize".to_string(),
            percent: 100,
            message: "Indexing complete".to_string(),
        },
    );

    let root = parsed
        .nodes
        .first()
        .ok_or_else(|| AppError::Internal("normalized payload contains no root node".to_string()))?;
    let section_count = parsed
        .nodes
        .iter()
        .filter(|node| {
            let kind = node.node_type.to_ascii_lowercase();
            kind == "section" || kind == "subsection"
        })
        .count();

    Ok(IngestDocumentResponse {
        document_id,
        root_node_id: root.id.clone(),
        node_count: parsed.nodes.len(),
        section_count,
    })
}

#[tauri::command]
pub async fn list_documents(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<ListDocumentsResponse> {
    let docs = documents::list_documents(state.db.pool(), &project_id).await?;
    Ok(ListDocumentsResponse { documents: docs })
}

#[tauri::command]
pub async fn open_document(
    state: State<'_, AppState>,
    document_id: String,
) -> AppResult<OpenDocumentResponse> {
    let document = documents::get_document(state.db.pool(), &document_id).await?;
    Ok(OpenDocumentResponse { document })
}

#[tauri::command]
pub async fn get_tree(
    state: State<'_, AppState>,
    document_id: String,
    parent_node_id: Option<String>,
    depth: Option<i64>,
) -> AppResult<GetTreeResponse> {
    let nodes = documents::get_tree(
        state.db.pool(),
        &document_id,
        parent_node_id.as_deref(),
        depth.unwrap_or(3),
    )
    .await?;
    Ok(GetTreeResponse { nodes })
}

#[tauri::command]
pub async fn get_project_tree(
    state: State<'_, AppState>,
    project_id: String,
    depth: Option<i64>,
) -> AppResult<GetTreeResponse> {
    let nodes = documents::get_project_tree(state.db.pool(), &project_id, depth.unwrap_or(3)).await?;
    Ok(GetTreeResponse { nodes })
}

#[tauri::command]
pub async fn get_node(state: State<'_, AppState>, node_id: String) -> AppResult<GetNodeResponse> {
    let node = documents::get_node(state.db.pool(), &node_id).await?;
    Ok(GetNodeResponse { node })
}

#[tauri::command]
pub async fn get_document_preview(
    state: State<'_, AppState>,
    document_id: String,
) -> AppResult<GetDocumentPreviewResponse> {
    let blocks = documents::get_document_preview(state.db.pool(), &document_id)
        .await?
        .into_iter()
        .map(|node| DocumentPreviewBlock {
            id: node.id,
            document_id: node.document_id,
            parent_id: node.parent_id,
            node_type: node.node_type,
            title: node.title,
            text: node.text,
            ordinal_path: node.ordinal_path,
        })
        .collect();

    Ok(GetDocumentPreviewResponse {
        document_id,
        blocks,
    })
}

#[tauri::command]
pub async fn get_graph_layout(
    state: State<'_, AppState>,
    document_id: String,
) -> AppResult<GetGraphLayoutResponse> {
    let positions = documents::get_graph_layout(state.db.pool(), &document_id).await?;
    Ok(GetGraphLayoutResponse {
        document_id,
        positions,
    })
}

#[tauri::command]
pub async fn save_graph_layout(
    state: State<'_, AppState>,
    document_id: String,
    positions: Vec<GraphNodePosition>,
) -> AppResult<SaveGraphLayoutResponse> {
    let saved = documents::save_graph_layout(state.db.pool(), &document_id, &positions).await?;
    Ok(SaveGraphLayoutResponse { saved })
}

#[tauri::command]
pub async fn export_markdown(
    state: State<'_, AppState>,
    document_id: String,
) -> AppResult<ExportMarkdownResponse> {
    let export_dir = state.data_dir.join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|err| AppError::Io(err.to_string()))?;
    let file_path = export_dir.join(format!("{document_id}.md"));
    documents::export_markdown(state.db.pool(), &document_id, &file_path).await?;
    Ok(ExportMarkdownResponse {
        file_path: file_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn delete_document(
    state: State<'_, AppState>,
    document_id: String,
) -> AppResult<DeleteDocumentResponse> {
    let deleted = documents::delete_document(state.db.pool(), &document_id).await?;
    Ok(DeleteDocumentResponse { deleted })
}
