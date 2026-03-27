use super::*;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[test]
fn parse_status_accepts_valid_score() {
    assert_eq!(parse_posture_status("7").unwrap(), PostureStatus::Score(7));
}

#[test]
fn parse_status_trims_whitespace() {
    assert_eq!(
        parse_posture_status(" 10 \n").unwrap(),
        PostureStatus::Score(10)
    );
}

#[test]
fn parse_status_supports_no_person_marker() {
    assert_eq!(parse_posture_status("N").unwrap(), PostureStatus::NoPerson);
    assert_eq!(parse_posture_status("n").unwrap(), PostureStatus::NoPerson);
}

#[test]
fn parse_status_rejects_invalid_values() {
    assert!(parse_posture_status("0").is_err());
    assert!(parse_posture_status("11").is_err());
    assert!(parse_posture_status("bad posture").is_err());
}

#[test]
fn parse_api_response_extracts_nested_content() {
    let response_json = json!({
        "choices": [{
            "message": {
                "content": "8"
            }
        }]
    });

    assert_eq!(
        parse_api_response(&response_json).unwrap(),
        PostureStatus::Score(8)
    );
}

#[test]
fn parse_api_response_supports_content_parts() {
    let response_json = json!({
        "choices": [{
            "message": {
                "content": [{ "type": "text", "text": "7" }]
            }
        }]
    });

    assert_eq!(
        parse_api_response(&response_json).unwrap(),
        PostureStatus::Score(7)
    );
}

#[test]
fn parse_status_accepts_number_with_extra_text() {
    assert_eq!(
        parse_posture_status("Posture score: 4").unwrap(),
        PostureStatus::Score(4)
    );
}

#[test]
fn parse_api_response_errors_when_content_missing() {
    let response_json = json!({"choices": []});
    assert!(parse_api_response(&response_json).is_err());
}

#[tokio::test]
async fn analyze_errors_when_api_key_missing() {
    let analyzer = PostureAnalyzer::new();
    let config = Config {
        api_key: String::new(),
        ..Config::default()
    };
    let err = analyzer
        .analyze(&[0xFF, 0xD8, 0xFF, 0xD9], &config)
        .await
        .expect_err("missing api key should return an error");
    assert!(err.to_string().contains("API key not configured"));
}

async fn spawn_one_shot_http_server(status: &str, body: &str) -> String {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    std::env::set_var("no_proxy", "127.0.0.1,localhost");
    std::env::set_var("HTTP_PROXY", "");
    std::env::set_var("HTTPS_PROXY", "");
    std::env::set_var("http_proxy", "");
    std::env::set_var("https_proxy", "");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let addr = listener.local_addr().expect("test server local addr");
    let status_line = status.to_string();
    let response_body = body.to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept request");
        let mut request = Vec::new();
        let mut buf = [0_u8; 1024];
        loop {
            let read = socket.read(&mut buf).await.expect("read request");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buf[..read]);
            if request.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }

        let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write response");
    });

    format!("http://{addr}/v1/chat/completions")
}

#[tokio::test]
async fn analyze_parses_successful_api_response() {
    let endpoint =
        spawn_one_shot_http_server("200 OK", r#"{"choices":[{"message":{"content":"9"}}]}"#).await;

    let analyzer = PostureAnalyzer::new();
    let config = Config {
        api_key: "test-key".to_string(),
        provider_endpoint: endpoint,
        ..Config::default()
    };

    let status = analyzer
        .analyze(&[0xFF, 0xD8, 0xFF, 0xD9], &config)
        .await
        .expect("successful response should parse");
    assert_eq!(status, PostureStatus::Score(9));
}

#[tokio::test]
async fn analyze_surfaces_non_success_http_errors() {
    let endpoint =
        spawn_one_shot_http_server("401 Unauthorized", r#"{"error":"bad api key"}"#).await;

    let analyzer = PostureAnalyzer::new();
    let config = Config {
        api_key: "test-key".to_string(),
        provider_endpoint: endpoint,
        ..Config::default()
    };

    let err = analyzer
        .analyze(&[0xFF, 0xD8, 0xFF, 0xD9], &config)
        .await
        .expect_err("non-success response should return error");
    assert!(err.to_string().contains("API error"));
}
