use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::head_bucket::HeadBucketError;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use image::ImageFormat;
use std::io::Cursor;
use uuid::Uuid;
use crate::errors::app_error::AppError;
use tracing::info;

pub struct StorageService;

impl StorageService {
    fn is_not_found(err: &SdkError<HeadBucketError>) -> bool {
        use aws_sdk_s3::error::ProvideErrorMetadata;
        match err {
            SdkError::ServiceError(context) => {
                matches!(context.err().code(), Some("NoSuchBucket") | Some("NotFound"))
            }
            _ => false,
        }
    }

    pub async fn ensure_bucket_exists(client: &Client, bucket: &str) -> Result<(), AppError> {
        let mut retry_count = 0;
        let max_retries = 5;
        let mut delay = tokio::time::Duration::from_secs(1);

        while retry_count < max_retries {
            tracing::info!("Checking if bucket '{}' exists...", bucket);
            
            match client.head_bucket().bucket(bucket).send().await {
                Ok(_) => {
                    tracing::info!("Bucket '{}' exists and is accessible.", bucket);
                    return Ok(());
                }
                Err(e) => {
                    if Self::is_not_found(&e) {
                        tracing::warn!("Bucket '{}' not found. Creating...", bucket);
                        
                        match client.create_bucket().bucket(bucket).send().await {
                            Ok(_) => {
                                tracing::info!("Bucket '{}' created successfully.", bucket);
                                
                                // Apply public read access to images
                                let policy = serde_json::json!({
                                    "Version": "2012-10-17",
                                    "Statement": [
                                        {
                                            "Effect": "Allow",
                                            "Principal": "*",
                                            "Action": ["s3:GetObject"],
                                            "Resource": [format!("arn:aws:s3:::{}/*", bucket)]
                                        }
                                    ]
                                });

                                if let Err(pe) = client.put_bucket_policy()
                                    .bucket(bucket)
                                    .policy(policy.to_string())
                                    .send()
                                    .await {
                                        tracing::error!("Failed to apply public policy to bucket '{}': {:?}", bucket, pe);
                                    } else {
                                        tracing::info!("Public read policy applied to bucket '{}'.", bucket);
                                    }
                                
                                return Ok(());
                            }
                            Err(ce) => {
                                tracing::error!("Failed to create bucket '{}' after 404: {:?}.", bucket, ce);
                                return Err(AppError::StorageError(format!("Critical failure creating bucket '{}'", bucket)));
                            }
                        }
                    } else {
                        tracing::error!("Transient error checking bucket '{}': {:?}. Retrying in {:?}...", bucket, e, delay);
                    }
                }
            }

            tokio::time::sleep(delay).await;
            retry_count += 1;
            delay *= 2; // Exponential backoff for transient errors
        }

        Err(AppError::StorageError(format!("Failed to ensure bucket '{}' exists after {} attempts", bucket, max_retries)))
    }

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
        let bucket_name = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "mikrotik-images".to_string());

        let byte_stream = ByteStream::from(compressed_bytes);
        
        if let Err(e) = client.put_object()
            .bucket(&bucket_name)
            .key(&file_name)
            .body(byte_stream)
            .content_type("image/webp")
            .send()
            .await {
                tracing::error!("FATAL: Storage service error during upload to bucket '{}': {:?}", bucket_name, e);
                return Err(AppError::StorageError(format!("Failed to upload to storage: service error. Check internal logs for details.")));
            }

        // 6. Return the public URL
        let endpoint = std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
        let public_url = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket_name, file_name);
        
        info!("Image successfully uploaded to: {}", public_url);
        Ok(public_url)

    }
}
