use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use std::env;

pub async fn connect() -> Client {
    let access_key = env::var("MINIO_ROOT_USER").expect("MINIO_ROOT_USER must be set");
    let secret_key = env::var("MINIO_ROOT_PASSWORD").expect("MINIO_ROOT_PASSWORD must be set");
    let endpoint_url = env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set");

    let credentials = Credentials::new(access_key, secret_key, None, None, "minio");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .endpoint_url(endpoint_url)
        .region(aws_config::Region::new("us-east-1"))
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    let client = Client::from_conf(s3_config);

    // Ensure mikrotik-images bucket exists
    let bucket_name = "mikrotik-images";
    if let Err(e) = client.head_bucket().bucket(bucket_name).send().await {
        let err_msg = e.to_string();
        if err_msg.contains("NotFound") || err_msg.contains("404") {
            tracing::info!("Bucket '{}' not found, creating it...", bucket_name);
            let _ = client.create_bucket().bucket(bucket_name).send().await;
            
            // Allow public read access to images
            let policy = serde_json::json!({
                "Version": "2012-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Principal": "*",
                        "Action": ["s3:GetObject"],
                        "Resource": [format!("arn:aws:s3:::{}/*", bucket_name)]
                    }
                ]
            });
            
            let _ = client.put_bucket_policy()
                .bucket(bucket_name)
                .policy(policy.to_string())
                .send()
                .await;
        }
    }

    client
}
