use az_logger::{Logger, LoggerOptions};

#[tokio::test]
async fn test_async_logger() {
    let opts = LoggerOptions {
        log_dir: Some("test_logs".to_string()),
        log_name_format: Some("async_test_<timestamp>.log".to_string()),
        ..Default::default()
    };
    Logger::init(None::<String>, opts).unwrap();
    Logger::log_info("hello async logger", file!(), line!());
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let files = tokio::fs::read_dir("test_logs").await.unwrap();
    tokio::pin!(files);
    while let Some(file) = files.next_entry().await.unwrap() {
        let contents = tokio::fs::read_to_string(file.path()).await.unwrap();
        println!("Contents: {}", &contents);
        if file.path().starts_with("async_test") {
            assert!(contents.contains("hello async logger"));
        }
    }
}