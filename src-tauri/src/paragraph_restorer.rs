use crate::model::Content;

/// 段落恢复器 - 从混乱的段落列表中恢复正确顺序
pub struct ParagraphRestorer {
    chapter_id: u64,
}

impl ParagraphRestorer {
    // 算法相关常量
    const KEEP_ORDER_THRESHOLD: usize = 20;
    // 0x14

    // 种子生成常量
    const SEED_MULTIPLIER: u64 = 0x7e;
    const SEED_OFFSET: u64 = 0xe8;
    const SHUFFLE_INCREMENT: u64 = 0xc0f5;
    const SHUFFLE_MODULUS: u64 = 0x38f40;
    // 洗牌算法常量
    const SHUFFLE_MULTIPLIER: u64 = 0x2456;

    pub fn get_version() -> &'static str {
        "chapterlog.js?v1006c1.3"
    }

    /// 创建新的段落恢复器
    ///
    /// # 参数
    /// * `chapter_id` - 章节ID
    pub fn new(chapter_id: u64) -> Self {
        Self { chapter_id }
    }

    /// 生成种子值
    pub fn generate_seed(chapter_id: u64) -> u64 {
        (chapter_id * Self::SEED_MULTIPLIER) + Self::SEED_OFFSET
    }

    /// 从字符串列表中恢复正确顺序
    ///
    /// 实现与JavaScript相同的逻辑：
    /// - 如果段落数量 <= 20，保持原顺序
    /// - 如果段落数量 > 20，前20个保持原顺序，后面的需要恢复
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
            .filter(|p| Self::is_reorder_target(p))
            .collect();

        let n = text_paragraphs.len();

        if n <= Self::KEEP_ORDER_THRESHOLD {
            // 段落数量较少，保持原顺序
            return paragraphs;
        }

        // 分割段落：前20个保持原顺序，后面的需要恢复
        let (keep_order, need_restore): (Vec<_>, Vec<_>) = text_paragraphs
            .into_iter()
            .enumerate()
            .partition(|(i, _)| *i < Self::KEEP_ORDER_THRESHOLD);

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
                if Self::is_reorder_target(&paragraphs[i]) {
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
            current_seed = (current_seed * Self::SHUFFLE_MULTIPLIER + Self::SHUFFLE_INCREMENT)
                % Self::SHUFFLE_MODULUS;
            let random_index =
                ((current_seed as f64 / Self::SHUFFLE_MODULUS as f64) * (i + 1) as f64) as usize;

            indices.swap(i, random_index);
        }

        // 根据恢复的索引重新排列段落
        let mut result = vec![Content::Text(String::new()); n];
        for (original_index, &correct_index) in indices.iter().enumerate() {
            result[correct_index] = paragraphs[original_index].clone();
        }

        result
    }

    fn is_reorder_target(content: &Content) -> bool {
        match content {
            Content::Text(text) => !text.trim().is_empty(),
            Content::Tag(tag) => {
                let trimmed = tag.trim_start();
                if !(trimmed.starts_with("<p") || trimmed.starts_with("<P")) {
                    return false;
                }

                let mut in_tag = false;
                for ch in tag.chars() {
                    match ch {
                        '<' => in_tag = true,
                        '>' => in_tag = false,
                        _ => {
                            if !in_tag && !ch.is_whitespace() {
                                return true;
                            }
                        }
                    }
                }

                false
            }
            Content::Image(_) => false,
        }
    }

    pub fn restore_with_index(text: Vec<Content>, index_list: Vec<usize>) -> Vec<Content> {
        let mut restored = vec![];
        for i in index_list {
            restored.push(text[i].clone());
        }
        restored
    }
}

/// 使用示例和测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client, config, parse};

    #[tokio::test]
    async fn test_paragraph_restorer() {
        let config = config::Config::default();

        let client = client::BiliClient::new(
            config.base_url.as_str(),
            config.cookie.as_str(),
            config.user_agent.as_str(),
            &config.headers,
            config.convert_simple_chinese,
        )
        .unwrap();

        let html = client
            .get("https://www.bilinovel.com/novel/1/2.html")
            .await
            .unwrap();

        let mut text = vec![];
        let mut img_list = vec![];
        parse::parse_novel_text(&html, &mut text, &mut img_list, &config.base_url);

        let restorer = ParagraphRestorer::new(2);
        // 恢复正确顺序
        let restored = restorer.restore(text);

        let expected = vec![
            "和那个人的发色一样",
            "看着沾满鲜血的手掌，我如此心想。",
            "红色——比自然的莓金色更加艳丽的鲜红发色。",
            "没错，那个人的美丽鲜红长发，就和我手上的鲜血同样颜色。",
            "兵藤一诚——这是我的名字。「一诚、一诚。」父母与学校的人都是这样叫我。",
            "目前是个高二生，正值歌颂青春的年纪。",
            "偶尔会听见不认识的学生说：「那个家伙就是一诚吧？」真不知道我的名字是有多么家喻户晓。",
            "其实我是风云人物？",
            "不，没有这回事。毕竟我涉嫌偷窥女子剑道社的社办，成了恶名昭彰的色狼。",
            "竟然怀疑我偷窥女子社团的社办。这种不知廉耻的事，我……",
            "对不起，我的确在现场。就是女子剑道社隔壁的仓库。我原本是打算从仓库墙上的小洞偷窥没错。",
            "可是我没有偷窥。谁叫松田和元滨一直不肯让出偷窥孔，那些家伙真是……",
            "光是听到那两个笨蛋兴奋地说些「呜喔喔喔！村山的胸部果然很大。」、「喔——片濑的腿超美的。」之类的话，我就快要不行了。",
            "我也很想看啊！可是因为有人似乎打算进来仓库，我们只好赶紧落跑。",
            "就在我每天投注热情在这种色色的事时，幸福突然降临到我身上。",
            "「请你和我交往。」",
            "有女生向我告白！",
            "真是青春啊。",
            "对没有女朋友的我面吾，那就像一阵风——一阵名为青春的酸甜之风……",
            "我人生之中第一个女朋友——名叫天野夕麻。是个拥有一头润泽黑发的纤瘦女生。",
            "她真的好可爱，我一见到她就对她一见钟情。",
            "眼前出现一名超级美少女，又对我说「兵藤同学！找喜欢你！请你和我交往！」任谁都会立刻答应吧？",
            "对于一个没女友期=年龄的男人来说，这真是再梦幻也不过的状况。跟别人说了就算得到「你是在说哪个恋爱游戏的情节？」反应也不奇怪，但是真的发生了！",
            "奇迹真的发生了！有人对我告白！还是美少女！",
            "我原本也以为这是不是什么整人企划，还再三怀疑她是否带了人等在后面，准备见证惩罚游戏。",
            "这也是没办法的事。直到那一天的那一刻，我都是个以为自己天生不受惹人爱之星眷顾的少年。",
            "从那天开始，我有了女朋友。层次整个不一样了。该怎么说，感觉心情十分轻松。在学校的走廊上和其他男同学擦身而过时，我都想对他们说……",
            "我赢了！",
        ];

        let actual: Vec<String> = restored
            .iter()
            .filter(|content| ParagraphRestorer::is_reorder_target(content))
            .take(expected.len())
            .map(extract_plain_text)
            .collect();

        assert_eq!(actual, expected, "段落恢复顺序不符合预期");
    }

    fn extract_plain_text(content: &Content) -> String {
        match content {
            Content::Text(text) => text.trim().to_string(),
            Content::Tag(tag) => {
                let mut in_tag = false;
                let mut result = String::new();

                for ch in tag.chars() {
                    match ch {
                        '<' => in_tag = true,
                        '>' => in_tag = false,
                        _ => {
                            if !in_tag {
                                result.push(ch);
                            }
                        }
                    }
                }

                result.trim().to_string()
            }
            Content::Image(_) => String::new(),
        }
    }
}
