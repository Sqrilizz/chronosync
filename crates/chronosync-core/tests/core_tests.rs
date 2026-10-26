use chronosync_core::{HeartbeatPayload, IngestResponse, detect_language};

#[test]
fn test_heartbeat_single_serialization() {
    let json_str = r#"{
        "project": "sqlxproxy",
        "branch": "main",
        "language": "rust",
        "file": "crates/core/src/network.rs",
        "editor": "zed",
        "timestamp": 1793041200,
        "is_write": true
    }"#;

    let payload: HeartbeatPayload = serde_json::from_str(json_str).expect("Valid json");
    let list = payload.into_vec();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].project, "sqlxproxy");
    assert_eq!(list[0].branch.as_deref(), Some("main"));
    assert_eq!(list[0].language.as_deref(), Some("rust"));
    assert_eq!(list[0].file.as_deref(), Some("crates/core/src/network.rs"));
    assert_eq!(list[0].editor.as_deref(), Some("zed"));
    assert_eq!(list[0].timestamp, 1793041200);
    assert!(list[0].is_write);
}

#[test]
fn test_heartbeat_batch_serialization() {
    let json_str = r#"[
        {
            "project": "sqlxproxy",
            "timestamp": 1793041200
        },
        {
            "project": "chronosync",
            "timestamp": 1793041260
        }
    ]"#;

    let payload: HeartbeatPayload = serde_json::from_str(json_str).expect("Valid batch json");
    let list = payload.into_vec();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].project, "sqlxproxy");
    assert_eq!(list[1].project, "chronosync");
}

#[test]
fn test_detect_language() {
    assert_eq!(detect_language("src/main.rs").as_deref(), Some("rust"));
    assert_eq!(detect_language("lib/app.ex").as_deref(), Some("elixir"));
    assert_eq!(detect_language("script.py").as_deref(), Some("python"));
    assert_eq!(detect_language("main.go").as_deref(), Some("go"));
    assert_eq!(detect_language("index.ts").as_deref(), Some("typescript"));
    assert_eq!(detect_language("unknown.xyz"), None);
}

#[test]
fn test_ingest_response() {
    let resp = IngestResponse {
        status: "ok".to_string(),
        accepted: 5,
        total: 5,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let decoded: IngestResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp, decoded);
}
