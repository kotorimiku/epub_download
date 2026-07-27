/// 消息回调特征别名
pub trait MessageCallback: Fn(&str) + Send + Sync {}
impl<F: Fn(&str) + Send + Sync> MessageCallback for F {}
