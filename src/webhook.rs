use reqwest::blocking::Response;
use serde_json::json;
use tracing::error;
use tracing::info;

#[tracing::instrument(skip_all)]
pub fn send(webhook_url: &str, message: &str) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    let params = json!({
        "content": message
    });
    let response: Response = client.post(webhook_url).json(&params).send()?;

    if response.status().is_success() {
        info!("Message successfully sent to webhoook");
    } else {
        let status = response.status();
        error!("Received response status: {status:?}");
        let body = response.text()?;
        error!("Response body: {body}");
    }

    Ok(())
}
