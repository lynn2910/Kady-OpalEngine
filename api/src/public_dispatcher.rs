use axum::body::StreamBody;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use hyper::header;
use log::error;
use tokio_util::io::ReaderStream;
use crate::AppState;

/// Handle every requests for public resources
pub async fn handler(file_name: Path<String>, State(app): State<AppState>) -> impl IntoResponse {
    let declared_files = app.public_files.read().await;

    let is_declared = declared_files.iter()
        .find(|(n, _, _)| { n == file_name.0.as_str() })
        .cloned();

    // free the lock
    drop(declared_files);

    if let Some((name, path, file_type)) = is_declared {
        // send the file :O

        let file = match tokio::fs::File::open(path).await {
            Ok(file) => file,
            Err(err) => {
                error!(target: "ApiHandler", "The API public file dispatcher returned an error for '{}': {err:#?}", file_name.as_str());
                return Err((StatusCode::NOT_FOUND, "Not found".to_string()))
            },
        };

        let stream = ReaderStream::new(file);
        let body = StreamBody::new(stream);

        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(file_type.as_str()).unwrap());
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(format!("attachment; filename=\"{name}\"").as_str()).unwrap()
        );


        Ok((StatusCode::OK, headers, body))
    } else {
        Err((StatusCode::NOT_FOUND, "Not found".to_string()))
    }
}