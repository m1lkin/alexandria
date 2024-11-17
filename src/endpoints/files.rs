use axum::{extract::{Multipart, Path}, response::{IntoResponse, Response}, http::StatusCode, debug_handler};
use serde::{Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::State;
use tokio::fs;
use crate::AppState;
use crate::db::{get_record, update_record};
use crate::structures::{File, Resource};

// Структура для ответа
#[derive(Serialize)]
pub struct UploadResponse {
    success: bool,
    message: String,
    file_paths: Vec<String>,
}

// Создаем путь к директории для конкретного поста
pub async fn ensure_post_directory(post_id: i64) -> std::io::Result<PathBuf> {
    let post_dir = PathBuf::from("uploads").join(post_id.to_string());
    fs::create_dir_all(&post_dir).await?;
    Ok(post_dir)
}

#[debug_handler]
// Обработчик загрузки файлов для конкретного поста
pub async fn upload_files_to_post(
    Path(post_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Response {
    let mut post = match get_record(&post_id, &state.client.database("alexandria").collection::<Resource>("posts")).await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "post doesn't exist".to_string(),
            )
                .into_response();
        }
        Ok(record) => record
    };

    // Проверяем и создаем директорию для поста
    let post_dir = match ensure_post_directory(post_id).await {
        Ok(dir) => dir,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create directory: {}", e),
            )
                .into_response();
        }
    };

    let mut uploaded_files = Vec::new();
    let mut had_errors = false;

    // Обрабатываем каждый файл в multipart-запросе
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => sanitize_filename::sanitize(name),
            None => continue,
        };

        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                had_errors = true;
                eprintln!("Error reading file data: {}", e);
                continue;
            }
        };

        // Создаем полный путь к файлу
        let file_path = post_dir.join(&file_name);

        // Сохраняем файл
        if let Err(e) = fs::write(&file_path, &data).await {
            had_errors = true;
            eprintln!("Error saving file: {}", e);
            continue;
        }
        
        post.files.push(File {
            filename: file_name.clone(),
            size: data.len().try_into().unwrap(),
        });
        
        // Добавляем путь к файлу в список успешно загруженных
        uploaded_files.push(format!("{}", file_name));
    }
    
    if let Err(e) = update_record(&post_id, &post, &state.client.database("alexandria").collection::<Resource>("posts")).await {
        return e.into_response();
    }

    // Формируем ответ
    let response = UploadResponse {
        success: !had_errors && !uploaded_files.is_empty(),
        message: if had_errors {
            "Some files failed to upload".to_string()
        } else if uploaded_files.is_empty() {
            "No files were uploaded".to_string()
        } else {
            "Files uploaded successfully".to_string()
        },
        file_paths: uploaded_files,
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

// Обработчик скачивания файла для конкретного поста
pub async fn download_post_file(
    Path((post_id, filename)): Path<(i64, String)>,
) -> Response {
    let file_path = PathBuf::from("uploads").join(&post_id.to_string()).join(&filename);

    match fs::read(&file_path).await {
        Ok(data) => {
            let headers = [
                ("Content-Type", "application/octet-stream"),
                (
                    "Content-Disposition",
                    &format!("attachment; filename=\"{}\"", filename),
                ),
            ];
            (StatusCode::OK, headers, data).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

// Получение списка файлов поста
pub async fn list_post_files(Path(post_id): Path<String>) -> Response {
    let post_dir = PathBuf::from("uploads").join(&post_id);

    match fs::read_dir(&post_dir).await {
        Ok(mut entries) => {
            let mut files = Vec::new();

            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_name) = entry.file_name().into_string() {
                    files.push(file_name);
                }
            }
            (StatusCode::OK, axum::Json(files)).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            "Post directory not found".to_string(),
        )
            .into_response(),
    }
}
