// use rust_bert::pipelines::sentiment::SentimentModel;
// use rust_bert::pipelines::sentence_embeddings::{SentenceEmbeddingsModel, SentenceEmbeddingsBuilder};
// use rust_bert::pipelines::ner::NERModel;
// use anyhow::Result;

// struct SemanticAnalyzer {
//     sentiment_model: SentimentModel,
//     embeddings_model: SentenceEmbeddingsModel,
//     ner_model: NERModel,
// }

// impl SemanticAnalyzer {
//     fn new() -> Result<Self> {
//         // 初始化各个模型
//         let sentiment_model = SentimentModel::new(Default::default())?;
//         let embeddings_model = SentenceEmbeddingsBuilder::remote(Default::default())
//             .create_model()?;
//         let ner_model = NERModel::new(Default::default())?;

//         Ok(Self {
//             sentiment_model,
//             embeddings_model,
//             ner_model,
//         })
//     }

//     // 情感分析
//     fn analyze_sentiment(&self, text: &str) -> Result<(f32, String)> {
//         let sentiment = self.sentiment_model.predict(&[text])?;
//         let score = sentiment[0].score;
//         let label = sentiment[0].label.clone();
//         Ok((score, label))
//     }

//     // 生成句子嵌入向量
//     fn get_embeddings(&self, text: &str) -> Result<Vec<f32>> {
//         let embeddings = self.embeddings_model.encode(&[text])?;
//         Ok(embeddings[0].clone())
//     }

//     // 命名实体识别
//     fn extract_entities(&self, text: &str) -> Result<Vec<(String, String)>> {
//         let entities = self.ner_model.predict(&[text])?;
//         let mut results = Vec::new();
        
//         for entity in entities[0].iter() {
//             results.push((entity.word.clone(), entity.entity.clone()));
//         }
        
//         Ok(results)
//     }

//     // 语义相似度计算
//     fn calculate_similarity(&self, text1: &str, text2: &str) -> Result<f32> {
//         let embeddings = self.embeddings_model.encode(&[text1, text2])?;
//         let similarity = cosine_similarity(&embeddings[0], &embeddings[1]);
//         Ok(similarity)
//     }
// }

// // 辅助函数：计算余弦相似度
// fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
//     let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
//     let norm1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
//     let norm2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
//     dot_product / (norm1 * norm2)
// }

// pub fn start() -> Result<()> {
//     let analyzer = SemanticAnalyzer::new()?;
    
//     // 示例用法
//     let text = "I really love this amazing book!";
    
//     // 1. 情感分析
//     let (sentiment_score, sentiment_label) = analyzer.analyze_sentiment(text)?;
//     println!("Sentiment: {} (score: {:.3})", sentiment_label, sentiment_score);
    
//     // 2. 实体识别
//     let entities = analyzer.extract_entities(text)?;
//     println!("Entities found:");
//     for (word, entity_type) in entities {
//         println!("- {} ({})", word, entity_type);
//     }
    
//     // 3. 语义相似度比较
//     let text2 = "This book is fantastic!";
//     let similarity = analyzer.calculate_similarity(text, text2)?;
//     println!("Semantic similarity: {:.3}", similarity);
    
//     Ok(())
// }


use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
pub fn start(content: &str, source: Language, target: Language) -> anyhow::Result<String> {
    let model = TranslationModelBuilder::new()
        .with_source_languages(vec![Language::English, Language::ChineseMandarin])
        .with_target_languages(vec![Language::English, Language::Spanish, Language::French, Language::ChineseMandarin])
        .create_model()?;
    // let input_text = "This is a sentence to be translated";
    // let output = model.translate(&[input_text], Language::English, Language::ChineseMandarin)?;
    let output = model.translate(&[content], source, target)?;
    // for sentence in &output {
    //     println!("{}", sentence);
    // }
    Ok(output[0].clone())
}