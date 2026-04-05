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

    Client::from_conf(s3_config)
}
