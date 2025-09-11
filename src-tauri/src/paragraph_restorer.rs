use crate::model::Content;

/// 段落恢复器 - 从混乱的段落列表中恢复正确顺序
pub struct ParagraphRestorer {
    chapter_id: u64,
}

impl ParagraphRestorer {
    /// 创建新的段落恢复器
    ///
    /// # 参数
    /// * `chapter_id` - 章节ID
    pub fn new(chapter_id: u64) -> Self {
        Self { chapter_id }
    }

    /// 从字符串列表中恢复正确顺序
    ///
    /// chapterlog.js?v1006a6
    /// 实现与JavaScript相同的逻辑：
    /// - 如果段落数量 <= 19，保持原顺序
    /// - 如果段落数量 > 19，前19个保持原顺序，后面的需要恢复
    ///
    /// # 参数
    /// * `paragraphs` - 混乱的段落列表
    ///
    /// # 返回
    /// * 正确顺序的段落列表
    pub fn restore(&self, mut paragraphs: Vec<Content>) -> Vec<Content> {
        if paragraphs.is_empty() {
            return paragraphs;
        }

        let text_paragraphs: Vec<_> = paragraphs
            .iter()
            .filter(|p| p.is_text() && !p.is_empty())
            .collect();

        let n = text_paragraphs.len();
        const KEEP_ORDER_THRESHOLD: usize = 19; // 0x13

        if n <= KEEP_ORDER_THRESHOLD {
            // 段落数量较少，保持原顺序
            return paragraphs;
        }

        // 分割段落：前19个保持原顺序，后面的需要恢复
        let (keep_order, need_restore): (Vec<_>, Vec<_>) = text_paragraphs
            .into_iter()
            .enumerate()
            .partition(|(i, _)| *i < KEEP_ORDER_THRESHOLD);

        // 只对需要恢复的部分进行排序
        let restored_part = self.restore_partial(
            need_restore
                .into_iter()
                .map(|(_, content)| content.clone())
                .collect(),
        );

        // 合并结果：保持原顺序的部分 + 恢复的部分
        let mut result = keep_order
            .into_iter()
            .map(|(_, content)| content.clone())
            .collect::<Vec<_>>();
        result.extend(restored_part);

        let mut i = 0;

        for p in result.iter_mut() {
            while i < paragraphs.len() {
                if paragraphs[i].is_text() && !paragraphs[i].is_empty() {
                    paragraphs[i] = p.to_owned();
                    i += 1;
                    break;
                }
                i += 1;
            }
        }

        paragraphs
    }

    /// 恢复部分段落的顺序（用于需要恢复的部分）
    fn restore_partial(&self, paragraphs: Vec<Content>) -> Vec<Content> {
        let n = paragraphs.len();
        if n <= 1 {
            return paragraphs;
        }

        let seed = Self::generate_seed(self.chapter_id);

        let mut indices: Vec<usize> = (0..n).collect();
        let mut current_seed = seed;

        // 使用Fisher-Yates洗牌算法
        // 从后往前恢复顺序
        for i in (1..n).rev() {
            current_seed = (current_seed * 0x2455 + 0xc091) % 0x38f40;
            let random_index = ((current_seed as f64 / 0x38f40 as f64) * (i + 1) as f64) as usize;

            indices.swap(i, random_index);
        }

        // 根据恢复的索引重新排列段落
        let mut result = vec![Content::Text(String::new()); n];
        for (original_index, &correct_index) in indices.iter().enumerate() {
            result[correct_index] = paragraphs[original_index].clone();
        }

        result
    }

    pub fn restore_with_index(text: Vec<Content>, index_list: Vec<usize>) -> Vec<Content> {
        let mut restored = vec![];
        for i in index_list {
            restored.push(text[i].clone());
        }
        restored
    }

    /// 生成种子值
    pub fn generate_seed(chapter_id: u64) -> u64 {
        (chapter_id * 0x89) + 0xe9
    }
}

/// 使用示例和测试
#[cfg(test)]
mod tests {
    use crate::{client, config, parse};

    use super::*;

    #[test]
    fn test_paragraph_restorer() {
        let config = config::Config::new();

        let client = client::BiliClient::new(
            config.base_url.as_str(),
            config.cookie.as_str(),
            config.user_agent.as_str(),
        )
        .unwrap();

        let html = client
            .get("https://www.bilinovel.com/novel/1/2.html")
            .unwrap();

        let mut text = vec![];
        let mut img_list = vec![];
        parse::parse_novel_text(&html, &mut text, &mut img_list, &config.base_url);

        let restorer = ParagraphRestorer::new(2);
        // 2 507
        println!("seed: {}", ParagraphRestorer::generate_seed(2));

        // 恢复正确顺序
        let restored = restorer.restore(text);

        println!("恢复后的顺序:");
        for (i, paragraph) in restored.iter().enumerate() {
            println!("{}. {:?}", i + 1, paragraph);
        }
    }
}
