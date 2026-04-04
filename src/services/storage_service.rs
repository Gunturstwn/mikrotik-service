use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use image::ImageFormat;
use std::io::Cursor;
use uuid::Uuid;
use crate::errors::app_error::AppError;
use tracing::info;

pub struct StorageService;

impl StorageService {
    pub async fn process_and_upload_image(
        client: &Client,
        original_bytes: &[u8],
    ) -> Result<String, AppError> {
        info!("Validating and processing image of size {} bytes", original_bytes.len());
        
        // 1. Load the image from bytes. This strictly validates that it's a real image, rejecting scripts.
        let img = image::load_from_memory(original_bytes)
            .map_err(|_| AppError::BadRequest("Invalid image format or corrupted file. Scripts/malicious files are not allowed.".to_string()))?;
        
        // 2. We can optionally resize the image if it's too large (e.g. > 1920x1080)
        // For a profile photo, 800x800 is more than enough
        let img = if img.width() > 800 || img.height() > 800 {
            img.resize(800, 800, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        // 3. Compress to WEBP
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::WebP)
            .map_err(|e| AppError::InternalServerError(format!("Failed to encode image to WEBP: {}", e)))?;
            
        let compressed_bytes = buffer.into_inner();
        
        // 4. Double check the generated size to strictly limit to < 5MB (just in case)
        if compressed_bytes.len() > 5 * 1024 * 1024 {
            return Err(AppError::BadRequest("Image too complex/large even after compression. Please upload a smaller image.".to_string()));
        }

        // 5. Upload to S3/MinIO
        let unique_id = Uuid::new_v4();
        let file_name = format!("{}.webp", unique_id);
        let bucket_name = "mikrotik-images";

        let byte_stream = ByteStream::from(compressed_bytes);
        
        client.put_object()
            .bucket(bucket_name)
            .key(&file_name)
            .body(byte_stream)
            .content_type("image/webp")
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to upload to storage: {}", e)))?;

        // 6. Return the public URL
        let endpoint = std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
        let public_url = format!("{}/{}/{}", endpoint, bucket_name, file_name);
        
        info!("Image successfully uploaded to: {}", public_url);
        Ok(public_url)
    }
}
