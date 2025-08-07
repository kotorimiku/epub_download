use anyhow::Result;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener};

#[derive(Clone)]
pub struct Event {
    pub app: AppHandle,
}

impl Event {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn message(&self, msg: &str) {
        self.app.emit("message", msg).unwrap();
    }

    pub fn html(&self, html: &str) -> Result<String> {
        self.app.emit("html", html).unwrap();
        let html_result = Arc::new(Mutex::new(String::new()));
        let html_result_clone = Arc::clone(&html_result);
        let (sender, receiver) = std::sync::mpsc::channel();
        let sender_clone = sender.clone();

        // 存储监听器ID，以便后续移除
        let listener_id = self.app.listen("restoreHtml", move |event| {
            let mut html_guard = html_result_clone.lock().unwrap();

            // 尝试解析JSON并获取原始字符串
            let payload_str = event.payload().to_string();
            let clean_str = if let std::result::Result::Ok(str) = serde_json::from_str(&payload_str)
            {
                str
            } else {
                // 如果不是JSON格式，直接使用原始字符串
                payload_str
            };

            *html_guard = clean_str;
            let _ = sender_clone.send(()); // 发送信号表示结果已准备好
        });

        // 等待结果
        receiver.recv().unwrap();

        // 移除监听器
        self.app.unlisten(listener_id);

        let result = html_result.lock().unwrap().clone();
        Ok(result)
    }
}
