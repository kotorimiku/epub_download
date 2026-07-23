use crate::error::Result;

/// 消息回调特征别名
pub trait MessageCallback: Fn(&str) + Send + Sync {}
impl<F: Fn(&str) + Send + Sync> MessageCallback for F {}

/// HTML 恢复回调特征别名
pub trait HtmlRestoreCallback: Fn(&str) -> Result<String> + Send + Sync {}
impl<H: Fn(&str) -> Result<String> + Send + Sync> HtmlRestoreCallback for H {}
