use actix_web::{web, get, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::fs;
use serde_json;
use anyhow::{Context, Result};
use actix_files::Files;

/// 单词数据结构，用于JSON序列化/反序列化
/// 包含词语(term)和含义(definition)两个字段
#[derive(Debug, Serialize, Deserialize)]
struct Word {
    term: String,
    definition: String,
}

/// 获取单词列表
/// 返回格式：`application/json`
#[get("/words")]
async fn get_words(data: web::Data<Vec<Word>>) -> impl Responder {
    HttpResponse::Ok().json(&data)
}

#[actix_web::main]
async fn main() -> Result<()> {
    // 初始化流程
    // 1. 加载JSON文件 -> 2. 解析数据结构 -> 3. 启动web服务器

    // 加载JSON文件
    let file_content = fs::read_to_string("words.json").context("读取words.json文件失败")?;
    // 反序列化JSON数据
    let words: Vec<Word> = serde_json::from_str(&file_content).context("解析words.json失败")?;
    let data = web::Data::new(words);

    // 启动web服务器
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(get_words)
            .service(Files::new("/", "./static/").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
    .context("启动服务器失败")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    /// 测试用例：验证Word结构的序列化/反序列化正确性
    /// 测试用例：验证/words返回正确的HTTP状态码和数据结构
    #[actix_web::test]
    async fn test_word_serialization() {
        let word = Word {
            term: "test".to_string(),
            definition: "测试".to_string(),
        };
        
        let json = serde_json::to_string(&word).unwrap();
        let decoded: Word = serde_json::from_str(&json).unwrap();
        
        assert_eq!(word.term, decoded.term);
        assert_eq!(word.definition, decoded.definition);
    }

    /// 测试用例：验证Word结构的序列化/反序列化正确性
    /// 测试用例：验证/words端点返回正确的HTTP状态码和数据结构
    #[actix_web::test]
    async fn test_get_words_endpoint() {
        let test_data = web::Data::new(vec![
            Word {
                term: "hello".to_string(),
                definition: "你好".to_string(),
            }
        ]);

        let app = test::init_service(
            App::new()
                .app_data(test_data.clone())
                .service(get_words)
        ).await;

        let req = test::TestRequest::get().uri("/words").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        
        let body: Vec<Word> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0].term, "hello");
    }
}
