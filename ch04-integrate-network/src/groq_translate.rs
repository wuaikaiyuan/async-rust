///! Groq Translation Module API call

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_ffi::c_str;
use std::ffi::CString;

pub fn translate(source_text: &str, source_lang: &str, target_lang: &str, country: &str, api_key: &str) -> String {
    // let args: (&str, &str, &str, &str) = (source_lang, target_lang, source_text, country,);

    // let r1 = translate_m1(args);
    // let result = match r1 {
    //     Ok(result) => result,
    //     Err(e) => {
    //         println!("Error: {}", e);
    //         return "".to_string();
    //     }
    // };
    // println!("r1: {}", result);

    // let r2 = translate_m2(args);
    // let result = match r2 {
    //     Ok(result) => result,
    //     Err(e) => {
    //         println!("Error: {}", e);
    //         return "".to_string();
    //     }
    // };
    // println!("r2: {}", result);
    println!("【r3】 原文：{}", source_text);
    let result = translate_local(source_text, source_lang, target_lang, country, api_key);
    println!("【r3】 译文: {}", result);
    print_separator_lines(2);

    return result;
}

fn print_separator_lines(num_lines: usize) {
    for _ in 0..num_lines {
        print!("------------------------");
    }
    print!("\n");
}

pub fn translate_m1(args: (&str, &str, &str, &str)) -> PyResult<String> {
    Python::with_gil(|py| {
        // 加载并执行Python脚本
        let utils_script = CString::new(include_str!("../scripts/utils.py")).unwrap();
        py.run(utils_script.as_c_str(), None, None)?;
        
        // 获取Python函数
        let translate_act = py.eval(c_str!("translate"), None, None)?;

        // 调用Python函数并获取结果
        
        let result: String = translate_act.call1(args)?.extract::<String>()?;
        
        Ok(result)
    })
}

pub fn translate_m2(args: (&str, &str, &str, &str)) -> PyResult<String> {
    Python::with_gil(|py| {
        // 加载Python模块
        let sys = py.import("sys")?;
        let current_dir = std::env::current_dir()?;
        let scripts_dir = current_dir.join("scripts");
        let scripts_path = scripts_dir.to_string_lossy().to_string();
        // 加载脚本路径到系统路径中，类似python中的：sys.path.append(scripts_path);
        sys.getattr("path")?.call_method1("append", (scripts_path,))?;

        // 导入Python模块
        let my_module = py.import("utils")?;

        // 调用Python函数
        let result: String = my_module.call_method1("translate", args)?.extract()?;

        Ok(result)
    })
}

pub fn translate_local(source_text: &str, source_lang: &str, target_lang: &str, country: &str, api_key: &str) -> String {
    let result: PyResult<String> = Python::with_gil(|py| {
        // 安装包
        let install_packages = [
            ("groq", "groq"),
            ("typing", "typing"),
            ("tiktoken", "tiktoken"),
            ("dotenv", "python-dotenv"),
            // ("icecream", "icecream"),
            ("langchain_text_splitters", "langchain-text-splitters"),
        ];
        
        for (module, package) in install_packages {
            let install_code = format!(
                r#"
import importlib
try:
    importlib.import_module("{}")
except ImportError:
    import pip
    pip.main(["install", "{}"])
                "#,
                module, package
            );
            
            let install_code_cstr = CString::new(install_code).unwrap();

            py.run(install_code_cstr.as_c_str(), None, None)?;
        }
        
        // 主要代码
        let main_code = format!(
            r#"
import os
from typing import List, Union

import tiktoken
from dotenv import load_dotenv
from langchain_text_splitters import RecursiveCharacterTextSplitter

from groq import Groq


load_dotenv()  # read local .env file
client = Groq(api_key="{}")

MAX_TOKENS_PER_CHUNK = (
    1000  # if text is more than this many tokens, we'll break it up into
)

# discrete chunks to translate one chunk at a time


def num_tokens_in_string(
    input_str: str, encoding_name: str = "cl100k_base"
) -> int:
    encoding = tiktoken.get_encoding(encoding_name)
    num_tokens = len(encoding.encode(input_str))
    return num_tokens

def get_completion(
    prompt: str,
    system_message: str = "You are a helpful assistant.",
    # model: str = "gpt-4-turbo",
    # llama-3.3-70b-versatile, gemma2-9b-it
    model: str = "llama-3.3-70b-versatile",
    temperature: float = 0.3,
    json_mode: bool = False,
) -> Union[str, dict]:
    if json_mode:
        response = client.chat.completions.create(
            model=model,
            temperature=temperature,
            top_p=1,
            response_format={{"type": "json_object"}},
            messages=[
                {{"role": "system", "content": system_message}},
                {{"role": "user", "content": prompt}},
            ],
        )
        return response.choices[0].message.content
    else:
        response = client.chat.completions.create(
            model=model,
            temperature=temperature,
            top_p=1,
            messages=[
                {{"role": "system", "content": system_message}},
                {{"role": "user", "content": prompt}},
            ],
        )
        return response.choices[0].message.content


def one_chunk_initial_translation(
    source_lang: str, target_lang: str, source_text: str
) -> str:    
    system_message = f"You are an expert linguist, specializing in translation from {{source_lang}} to {{target_lang}}."

    translation_prompt = f"""This is an {{source_lang}} to {{target_lang}} translation, please provide the {{target_lang}} translation for this text. \
Do not provide any explanations or text apart from the translation.
{{source_lang}}: {{source_text}}

{{target_lang}}:"""

    translation = get_completion(translation_prompt, system_message=system_message)

    return translation


def one_chunk_reflect_on_translation(
    source_lang: str,
    target_lang: str,
    source_text: str,
    translation_1: str,
    country: str = "",
) -> str:
    system_message = f"You are an expert linguist specializing in translation from {{source_lang}} to {{target_lang}}. \
You will be provided with a source text and its translation and your goal is to improve the translation."

    if country != "":
        reflection_prompt = f"""Your task is to carefully read a source text and a translation from {{source_lang}} to {{target_lang}}, and then give constructive criticism and helpful suggestions to improve the translation. \
The final style and tone of the translation should match the style of {{target_lang}} colloquially spoken in {{country}}.

The source text and initial translation, delimited by XML tags <SOURCE_TEXT></SOURCE_TEXT> and <TRANSLATION></TRANSLATION>, are as follows:

<SOURCE_TEXT>
{{source_text}}
</SOURCE_TEXT>

<TRANSLATION>
{{translation_1}}
</TRANSLATION>

When writing suggestions, pay attention to whether there are ways to improve the translation's \n\
(i) accuracy (by correcting errors of addition, mistranslation, omission, or untranslated text),\n\
(ii) fluency (by applying {{target_lang}} grammar, spelling and punctuation rules, and ensuring there are no unnecessary repetitions),\n\
(iii) style (by ensuring the translations reflect the style of the source text and take into account any cultural context),\n\
(iv) terminology (by ensuring terminology use is consistent and reflects the source text domain; and by only ensuring you use equivalent idioms {{target_lang}}).\n\

Write a list of specific, helpful and constructive suggestions for improving the translation.
Each suggestion should address one specific part of the translation.
Output only the suggestions and nothing else."""

    else:
        reflection_prompt = f"""Your task is to carefully read a source text and a translation from {{source_lang}} to {{target_lang}}, and then give constructive criticisms and helpful suggestions to improve the translation. \

The source text and initial translation, delimited by XML tags <SOURCE_TEXT></SOURCE_TEXT> and <TRANSLATION></TRANSLATION>, are as follows:

<SOURCE_TEXT>
{{source_text}}
</SOURCE_TEXT>

<TRANSLATION>
{{translation_1}}
</TRANSLATION>

When writing suggestions, pay attention to whether there are ways to improve the translation's \n\
(i) accuracy (by correcting errors of addition, mistranslation, omission, or untranslated text),\n\
(ii) fluency (by applying {{target_lang}} grammar, spelling and punctuation rules, and ensuring there are no unnecessary repetitions),\n\
(iii) style (by ensuring the translations reflect the style of the source text and take into account any cultural context),\n\
(iv) terminology (by ensuring terminology use is consistent and reflects the source text domain; and by only ensuring you use equivalent idioms {{target_lang}}).\n\

Write a list of specific, helpful and constructive suggestions for improving the translation.
Each suggestion should address one specific part of the translation.
Output only the suggestions and nothing else."""

    reflection = get_completion(reflection_prompt, system_message=system_message)
    return reflection


def one_chunk_improve_translation(
    source_lang: str,
    target_lang: str,
    source_text: str,
    translation_1: str,
    reflection: str,
) -> str:    
    system_message = f"You are an expert linguist, specializing in translation editing from {{source_lang}} to {{target_lang}}."

    prompt = f"""Your task is to carefully read, then edit, a translation from {{source_lang}} to {{target_lang}}, taking into
account a list of expert suggestions and constructive criticisms.

The source text, the initial translation, and the expert linguist suggestions are delimited by XML tags <SOURCE_TEXT></SOURCE_TEXT>, <TRANSLATION></TRANSLATION> and <EXPERT_SUGGESTIONS></EXPERT_SUGGESTIONS> \
as follows:

<SOURCE_TEXT>
{{source_text}}
</SOURCE_TEXT>

<TRANSLATION>
{{translation_1}}
</TRANSLATION>

<EXPERT_SUGGESTIONS>
{{reflection}}
</EXPERT_SUGGESTIONS>

Please take into account the expert suggestions when editing the translation. Edit the translation by ensuring:

(i) accuracy (by correcting errors of addition, mistranslation, omission, or untranslated text),
(ii) fluency (by applying {{target_lang}} grammar, spelling and punctuation rules and ensuring there are no unnecessary repetitions), \
(iii) style (by ensuring the translations reflect the style of the source text)
(iv) terminology (inappropriate for context, inconsistent use), or
(v) other errors.

Output only the new translation and nothing else."""

    translation_2 = get_completion(prompt, system_message)

    return translation_2


def one_chunk_translate_text(
    source_lang: str, target_lang: str, source_text: str, country: str = ""
) -> str:    
    translation_1 = one_chunk_initial_translation(
        source_lang, target_lang, source_text
    )

    reflection = one_chunk_reflect_on_translation(
        source_lang, target_lang, source_text, translation_1, country
    )
    translation_2 = one_chunk_improve_translation(
        source_lang, target_lang, source_text, translation_1, reflection
    )

    return translation_2


def multichunk_initial_translation(
    source_lang: str, target_lang: str, source_text_chunks: List[str]
) -> List[str]:
    system_message = f"You are an expert linguist, specializing in translation from {{source_lang}} to {{target_lang}}."

    translation_prompt = """Your task is to provide a professional translation from {{source_lang}} to {{target_lang}} of PART of a text.

The source text is below, delimited by XML tags <SOURCE_TEXT> and </SOURCE_TEXT>. Translate only the part within the source text
delimited by <TRANSLATE_THIS> and </TRANSLATE_THIS>. You can use the rest of the source text as context, but do not translate any
of the other text. Do not output anything other than the translation of the indicated part of the text.

<SOURCE_TEXT>
{{tagged_text}}
</SOURCE_TEXT>

To reiterate, you should translate only this part of the text, shown here again between <TRANSLATE_THIS> and </TRANSLATE_THIS>:
<TRANSLATE_THIS>
{{chunk_to_translate}}
</TRANSLATE_THIS>

Output only the translation of the portion you are asked to translate, and nothing else.
"""

    translation_chunks = []
    for i in range(len(source_text_chunks)):
        # Will translate chunk i
        tagged_text = (
            "".join(source_text_chunks[0:i])
            + "<TRANSLATE_THIS>"
            + source_text_chunks[i]
            + "</TRANSLATE_THIS>"
            + "".join(source_text_chunks[i + 1 :])
        )

        prompt = translation_prompt.format(
            source_lang=source_lang,
            target_lang=target_lang,
            tagged_text=tagged_text,
            chunk_to_translate=source_text_chunks[i],
        )

        translation = get_completion(prompt, system_message=system_message)
        translation_chunks.append(translation)

    return translation_chunks


def multichunk_reflect_on_translation(
    source_lang: str,
    target_lang: str,
    source_text_chunks: List[str],
    translation_1_chunks: List[str],
    country: str = "",
) -> List[str]:
    system_message = f"You are an expert linguist specializing in translation from {{source_lang}} to {{target_lang}}. \
You will be provided with a source text and its translation and your goal is to improve the translation."

    if country != "":
        reflection_prompt = """Your task is to carefully read a source text and part of a translation of that text from {{source_lang}} to {{target_lang}}, and then give constructive criticism and helpful suggestions for improving the translation.
The final style and tone of the translation should match the style of {{target_lang}} colloquially spoken in {{country}}.

The source text is below, delimited by XML tags <SOURCE_TEXT> and </SOURCE_TEXT>, and the part that has been translated
is delimited by <TRANSLATE_THIS> and </TRANSLATE_THIS> within the source text. You can use the rest of the source text
as context for critiquing the translated part.

<SOURCE_TEXT>
{{tagged_text}}
</SOURCE_TEXT>

To reiterate, only part of the text is being translated, shown here again between <TRANSLATE_THIS> and </TRANSLATE_THIS>:
<TRANSLATE_THIS>
{{chunk_to_translate}}
</TRANSLATE_THIS>

The translation of the indicated part, delimited below by <TRANSLATION> and </TRANSLATION>, is as follows:
<TRANSLATION>
{{translation_1_chunk}}
</TRANSLATION>

When writing suggestions, pay attention to whether there are ways to improve the translation's:\n\
(i) accuracy (by correcting errors of addition, mistranslation, omission, or untranslated text),\n\
(ii) fluency (by applying {{target_lang}} grammar, spelling and punctuation rules, and ensuring there are no unnecessary repetitions),\n\
(iii) style (by ensuring the translations reflect the style of the source text and take into account any cultural context),\n\
(iv) terminology (by ensuring terminology use is consistent and reflects the source text domain; and by only ensuring you use equivalent idioms {{target_lang}}).\n\

Write a list of specific, helpful and constructive suggestions for improving the translation.
Each suggestion should address one specific part of the translation.
Output only the suggestions and nothing else."""

    else:
        reflection_prompt = """Your task is to carefully read a source text and part of a translation of that text from {{source_lang}} to {{target_lang}}, and then give constructive criticism and helpful suggestions for improving the translation.

The source text is below, delimited by XML tags <SOURCE_TEXT> and </SOURCE_TEXT>, and the part that has been translated
is delimited by <TRANSLATE_THIS> and </TRANSLATE_THIS> within the source text. You can use the rest of the source text
as context for critiquing the translated part.

<SOURCE_TEXT>
{{tagged_text}}
</SOURCE_TEXT>

To reiterate, only part of the text is being translated, shown here again between <TRANSLATE_THIS> and </TRANSLATE_THIS>:
<TRANSLATE_THIS>
{{chunk_to_translate}}
</TRANSLATE_THIS>

The translation of the indicated part, delimited below by <TRANSLATION> and </TRANSLATION>, is as follows:
<TRANSLATION>
{{translation_1_chunk}}
</TRANSLATION>

When writing suggestions, pay attention to whether there are ways to improve the translation's:\n\
(i) accuracy (by correcting errors of addition, mistranslation, omission, or untranslated text),\n\
(ii) fluency (by applying {{target_lang}} grammar, spelling and punctuation rules, and ensuring there are no unnecessary repetitions),\n\
(iii) style (by ensuring the translations reflect the style of the source text and take into account any cultural context),\n\
(iv) terminology (by ensuring terminology use is consistent and reflects the source text domain; and by only ensuring you use equivalent idioms {{target_lang}}).\n\

Write a list of specific, helpful and constructive suggestions for improving the translation.
Each suggestion should address one specific part of the translation.
Output only the suggestions and nothing else."""

    reflection_chunks = []
    for i in range(len(source_text_chunks)):
        # Will translate chunk i
        tagged_text = (
            "".join(source_text_chunks[0:i])
            + "<TRANSLATE_THIS>"
            + source_text_chunks[i]
            + "</TRANSLATE_THIS>"
            + "".join(source_text_chunks[i + 1 :])
        )
        if country != "":
            prompt = reflection_prompt.format(
                source_lang=source_lang,
                target_lang=target_lang,
                tagged_text=tagged_text,
                chunk_to_translate=source_text_chunks[i],
                translation_1_chunk=translation_1_chunks[i],
                country=country,
            )
        else:
            prompt = reflection_prompt.format(
                source_lang=source_lang,
                target_lang=target_lang,
                tagged_text=tagged_text,
                chunk_to_translate=source_text_chunks[i],
                translation_1_chunk=translation_1_chunks[i],
            )

        reflection = get_completion(prompt, system_message=system_message)
        reflection_chunks.append(reflection)

    return reflection_chunks


def multichunk_improve_translation(
    source_lang: str,
    target_lang: str,
    source_text_chunks: List[str],
    translation_1_chunks: List[str],
    reflection_chunks: List[str],
) -> List[str]:
    system_message = f"You are an expert linguist, specializing in translation editing from {{source_lang}} to {{target_lang}}."

    improvement_prompt = """Your task is to carefully read, then improve, a translation from {{source_lang}} to {{target_lang}}, taking into
account a set of expert suggestions and constructive criticisms. Below, the source text, initial translation, and expert suggestions are provided.

The source text is below, delimited by XML tags <SOURCE_TEXT> and </SOURCE_TEXT>, and the part that has been translated
is delimited by <TRANSLATE_THIS> and </TRANSLATE_THIS> within the source text. You can use the rest of the source text
as context, but need to provide a translation only of the part indicated by <TRANSLATE_THIS> and </TRANSLATE_THIS>.

<SOURCE_TEXT>
{{tagged_text}}
</SOURCE_TEXT>

To reiterate, only part of the text is being translated, shown here again between <TRANSLATE_THIS> and </TRANSLATE_THIS>:
<TRANSLATE_THIS>
{{chunk_to_translate}}
</TRANSLATE_THIS>

The translation of the indicated part, delimited below by <TRANSLATION> and </TRANSLATION>, is as follows:
<TRANSLATION>
{{translation_1_chunk}}
</TRANSLATION>

The expert translations of the indicated part, delimited below by <EXPERT_SUGGESTIONS> and </EXPERT_SUGGESTIONS>, are as follows:
<EXPERT_SUGGESTIONS>
{{reflection_chunk}}
</EXPERT_SUGGESTIONS>

Taking into account the expert suggestions rewrite the translation to improve it, paying attention
to whether there are ways to improve the translation's

(i) accuracy (by correcting errors of addition, mistranslation, omission, or untranslated text),
(ii) fluency (by applying {{target_lang}} grammar, spelling and punctuation rules and ensuring there are no unnecessary repetitions), \
(iii) style (by ensuring the translations reflect the style of the source text)
(iv) terminology (inappropriate for context, inconsistent use), or
(v) other errors.

Output only the new translation of the indicated part and nothing else."""

    translation_2_chunks = []
    for i in range(len(source_text_chunks)):
        # Will translate chunk i
        tagged_text = (
            "".join(source_text_chunks[0:i])
            + "<TRANSLATE_THIS>"
            + source_text_chunks[i]
            + "</TRANSLATE_THIS>"
            + "".join(source_text_chunks[i + 1 :])
        )

        prompt = improvement_prompt.format(
            source_lang=source_lang,
            target_lang=target_lang,
            tagged_text=tagged_text,
            chunk_to_translate=source_text_chunks[i],
            translation_1_chunk=translation_1_chunks[i],
            reflection_chunk=reflection_chunks[i],
        )

        translation_2 = get_completion(prompt, system_message=system_message)
        translation_2_chunks.append(translation_2)

    return translation_2_chunks


def multichunk_translation(
    source_lang, target_lang, source_text_chunks, country: str = ""
):
    translation_1_chunks = multichunk_initial_translation(
        source_lang, target_lang, source_text_chunks
    )

    reflection_chunks = multichunk_reflect_on_translation(
        source_lang,
        target_lang,
        source_text_chunks,
        translation_1_chunks,
        country,
    )

    translation_2_chunks = multichunk_improve_translation(
        source_lang,
        target_lang,
        source_text_chunks,
        translation_1_chunks,
        reflection_chunks,
    )

    return translation_2_chunks


def calculate_chunk_size(token_count: int, token_limit: int) -> int:    
    if token_count <= token_limit:
        return token_count

    num_chunks = (token_count + token_limit - 1) // token_limit
    chunk_size = token_count // num_chunks

    remaining_tokens = token_count % token_limit
    if remaining_tokens > 0:
        chunk_size += remaining_tokens // num_chunks

    return chunk_size


def translate(
    source_lang,
    target_lang,
    source_text,
    country,
    max_tokens=MAX_TOKENS_PER_CHUNK,
):
    num_tokens_in_text = num_tokens_in_string(source_text)

    print(f"token_size: {{num_tokens_in_text}}")

    if num_tokens_in_text < max_tokens:
        print("Translating text as a single chunk")


        final_translation = one_chunk_translate_text(
            source_lang, target_lang, source_text, country
        )

        return final_translation

    else:
        print("Translating text as a multiple chunk")

        token_size = calculate_chunk_size(
            token_count=num_tokens_in_text, token_limit=max_tokens
        )

        print(f"token_size: {{token_size}}")

        text_splitter = RecursiveCharacterTextSplitter.from_tiktoken_encoder(
            model_name="gpt-4",
            chunk_size=token_size,
            chunk_overlap=0,
        )

        source_text_chunks = text_splitter.split_text(source_text)

        translation_2_chunks = multichunk_translation(
            source_lang, target_lang, source_text_chunks, country
        )

        return "".join(translation_2_chunks)    
            "#,
            api_key
            // api_key, source_lang, target_lang, source_text, country
        );
        
        let main_code_cstr = CString::new(main_code).unwrap();
        py.run(main_code_cstr.as_c_str(), None, None)?;
        let args: (&str, &str, &str, &str) = (source_lang, target_lang, source_text, country,);
        let translate_act = py.eval(c_str!("translate"), None, None)?;
        let result: String = translate_act.call1(args)?.extract::<String>()?;

        // let locals = PyDict::new(py);

        // let main_code_cstr = CString::new(main_code).unwrap();
        // py.run(main_code_cstr.as_c_str(), None, Some(&locals))?;
        
        // let result = locals.get_item("result")?
        //     .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No result found"))?
        //     .extract::<String>()?;

        Ok(result)
    });

    match result {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Translate Error: {:?}", err);
            String::new()
        },
    }
    
}

