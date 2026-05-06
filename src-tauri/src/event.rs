use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener};

use crate::{bail, error::Result};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HtmlEventPayload {
    request_id: String,
    html: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreHtmlPayload {
    request_id: String,
    html: String,
}

pub fn message(app_handle: &AppHandle, msg: &str) {
    app_handle.emit("message", msg).unwrap();
}

pub fn print(app_handle: &AppHandle, msg: &str) {
    app_handle.emit("print", msg).unwrap();
}

pub fn html(app_handle: &AppHandle, html: &str) -> Result<String> {
    let request_id = format!(
        "{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );

    let html_result = Arc::new(Mutex::new(String::new()));
    let html_result_clone = Arc::clone(&html_result);
    let (sender, receiver) = std::sync::mpsc::channel();
    let sender_clone = sender.clone();
    let request_id_clone = request_id.clone();

    // 存储监听器ID，以便后续移除
    let listener_id = app_handle.listen("restoreHtml", move |event| {
        let payload_str = event.payload().to_string();
        let payload: std::result::Result<RestoreHtmlPayload, _> =
            serde_json::from_str(&payload_str);

        let std::result::Result::Ok(payload) = payload else {
            return;
        };

        if payload.request_id != request_id_clone {
            return;
        }

        let mut html_guard = html_result_clone.lock().unwrap();
        *html_guard = payload.html;

        let _ = sender_clone.send(()); // 发送信号表示结果已准备好
    });

    app_handle.emit(
        "html",
        HtmlEventPayload {
            request_id,
            html: html.to_string(),
        },
    )?;

    // 等待结果
    let wait_result = receiver.recv_timeout(Duration::from_secs(100));

    // 移除监听器
    app_handle.unlisten(listener_id);

    if let Err(err) = wait_result {
        message(app_handle, html);
        bail!("Failed to receive HTML result: {}", err);
    }

    let result = html_result.lock().unwrap().clone();
    Ok(result)
}
