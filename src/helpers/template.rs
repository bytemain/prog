use std::collections::HashMap;

/// 渲染模板字符串，将模板中的占位符替换为实际值。
///
/// # 参数
///
/// * `text` - 包含占位符的模板字符串。
/// * `values` - 一个哈希表，包含占位符的键值对。
///
/// # 返回值
///
/// 渲染后的字符串。
pub fn render_template(text: String, values: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut iter = text.chars().peekable();

    // 遍历输入的字符串
    while let Some(c) = iter.next() {
        if c == '{' && iter.peek() == Some(&'{') {
            iter.next(); // Skip the second '{'
            let mut key = String::new();
            // 收集占位符的键
            while let Some(c) = iter.next() {
                if c == '}' && iter.peek() == Some(&'}') {
                    iter.next(); // Skip the second '}'
                                 // 如果键存在于哈希表中，则替换占位符
                    if let Some(value) = values.get(&key) {
                        result.push_str(value);
                    } else {
                        // 如果键不存在，则保留占位符
                        result.push_str(&format!("{{{{{}}}}}", key));
                    }
                    break;
                } else {
                    key.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 测试正常渲染。
    #[test]
    fn test_render_template() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());
        values.insert("age".to_string(), "30".to_string());

        let template = "Hello, {{name}}! You are {{age}} years old.".to_string();
        let expected = "Hello, Alice! You are 30 years old.".to_string();

        assert_eq!(render_template(template, &values), expected);
    }

    /// 测试当键不存在时的渲染。
    #[test]
    fn test_render_template_missing_key() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());

        let template = "Hello, {{name}}! You are {{age}} years old.".to_string();
        let expected = "Hello, Alice! You are {{age}} years old.".to_string();

        assert_eq!(render_template(template, &values), expected);
    }
}
