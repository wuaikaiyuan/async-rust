use ch04_integrate_network::{
    hyper_start, mio_start, 
    semantic_start, srt_fmt_start, 
    dotenv_config, toml_config, get_groq_key_by_dotenv, get_groq_key_by_toml,
    translate_start
};
use rust_bert::pipelines::translation::Language;
use dotenv::dotenv;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // 加载 .env 文件中的环境变量

    // hyper_start()?;
    // mio_start()?;
    srt_fmt_start()?;

    // let mut api_key_dotenv = get_groq_key_by_dotenv();
    // println!("dotenv - GROQ_API_KEY: {:?}", api_key_dotenv);
    // let mut api_key_toml = get_groq_key_by_toml();
    // println!("toml - GROQ_API_KEY: {:?}", api_key_toml);
    
    // let source_text = "What is the capital of France?Rust Essentials A quick guide to writing fast, safe, and concurrent systems and applications (Ivo Balbaert) (Z-Library)";
    // let source_lang = "English";
    // let target_lang = "Chinese";
    // let country = "China";

    // let result = translate_start(source_text, source_lang, target_lang, country, 
    //     api_key_dotenv.unwrap_or(api_key_toml.expect("Failed to get GROQ API key")).as_str());
    // println!("source_text: {:?} \r\n result: {:?}", source_text, result);
    

    Ok(())
}


