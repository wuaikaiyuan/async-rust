use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use rust_bert::pipelines::translation::Language;
use crate::{semantic_start, translate_start, get_groq_key_by_dotenv, get_groq_key_by_toml};


#[derive(Debug)]
struct SubtitleLine {
    number: String,
    timestamp: String,
    content: String,
    transed_content: String,
    blank_line: String,
}

impl SubtitleLine {
    fn new() -> Self {
        SubtitleLine {
            number: String::new(),
            timestamp: String::new(),
            content: String::new(),
            transed_content: String::new(),
            blank_line: String::new(),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n",
            self.number, self.timestamp, self.content, self.transed_content, self.blank_line
        )
    }
}

fn is_timestamp(line: &str) -> bool {
    line.contains("-->") && line.contains(":") && line.contains(",")
}

fn is_number(line: &str) -> bool {
    line.trim().parse::<u32>().is_ok()
}

/*
    1. 读取srt文件
    2. 创建结构体类型：
        struct SubtitleLine {
            number: String,
            timestamp: String,
            content: String,
            blank_line: String
        }
    3. 遍历文件的每一行，判断当前行的类型
        - number，数字，跳过
        - timestamp，时间戳，跳过
        - content，文本内容，合并到一行
        - blank_line，空白行，跳过
        将如上封装到结构体 SubtitleLine 中，并添加到数组中

    4. 处理数组元素 SubtitleLine 中的每个属性转换为字符串，作为新的一行写入到新的文件中
*/
fn process_srt_file(input_path: &str, output_path: &str) -> anyhow::Result<()> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let mut subtitles: Vec<SubtitleLine> = Vec::new();
    let mut current_subtitle = SubtitleLine::new();
    let mut current_content = String::new();

    for line in reader.lines() {
        let line = line?;
        
        if is_number(&line) {
            if !current_content.is_empty() {
                current_subtitle.content = current_content.trim().to_string();
                // current_subtitle.transed_content = 
                //     semantic_start(current_content.trim(), Language::English, Language::ChineseMandarin)?;
                subtitles.push(current_subtitle);

                current_subtitle = SubtitleLine::new();
                current_content.clear();
            }
            current_subtitle.number = line;
        } else if is_timestamp(&line) {
            current_subtitle.timestamp = line;
        } else if line.trim().is_empty() {
            current_subtitle.blank_line = line;
        } else {
            if !current_content.is_empty() {
                current_content.push(' ');
            }
            current_content.push_str(&line.trim());
            
            current_subtitle.content = current_content.trim().to_string();
        }
    }

    if !current_content.is_empty() {
        current_subtitle.content = current_content.trim().to_string();
        // current_subtitle.transed_content = semantic_start(current_content.trim(), Language::English, Language::ChineseMandarin)?;
        subtitles.push(current_subtitle);
    }

    let source_lang = "English";
    let target_lang = "Chinese";
    let country = "China";

    // 翻译
    for subtitle in &mut subtitles {
        // subtitle.transed_content = semantic_start(&subtitle.content, Language::English, Language::ChineseMandarin)?;
        // 翻译
        let result = translate_start(&subtitle.content, source_lang, target_lang, country,
            get_groq_key_by_dotenv().unwrap_or(get_groq_key_by_toml().expect("Failed to get GROQ API key")).as_str());
        subtitle.transed_content = result;
    }

    // println!("{:?}", subtitles);

    // Write to output file
    let mut output_file = File::create(output_path)?;
    for subtitle in subtitles {
        output_file.write_all(subtitle.to_string().as_bytes())?;
    }

    Ok(())
}

pub fn start() -> anyhow::Result<()> {
    let input_path = "i:/TRACTOR.srt";
    let output_path = "i:/TRACTOR_formatted.srt";

    process_srt_file(input_path, output_path)?;
    println!("SRT file has been processed successfully!");
    
    Ok(())
}