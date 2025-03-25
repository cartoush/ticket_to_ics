use std::{
    env::args,
    io::{BufWriter, Cursor},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    fmt_message, fmt_template,
    llm::ollama::client::Ollama,
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::Message,
    template_fstring,
};
use pdf2image::{PDF, RenderOptionsBuilder, image::ImageFormat};

#[async_std::main]
async fn main() {
    let path = args().nth(1).expect("no file given");
    println!("PDF filepath: {}", path);

    let pdf = PDF::from_file(&path).unwrap();
    let pages = pdf
        .render(
            pdf2image::Pages::Range(1..=8),
            RenderOptionsBuilder::default().build().unwrap(),
        )
        .unwrap();

    let mut writer = BufWriter::new(Cursor::new(Vec::new()));
    pages[0].write_to(&mut writer, ImageFormat::Jpeg).unwrap();

    let buf = writer.into_inner().unwrap().into_inner();
    let image_b64 = BASE64_STANDARD.encode(&buf);

    let prompt = message_formatter![
        fmt_message!(Message::new_system_message(
            "A chat between a curious user and an artificial intelligence assistant. The assistant gives helpful, detailed, and polite answers to the user's questions."
        )),
        fmt_message!(Message::new_human_message_with_images(vec![image_b64])),
        fmt_template!(HumanMessagePromptTemplate::new(template_fstring!(
            "{input}", "input"
        ))),
    ];

    // let ollama = Ollama::default().with_model("llama3.2-vision");
    let ollama = Ollama::default().with_model("granite3.2-vision:2b-q8_0");
    let chain = LLMChainBuilder::new()
        .prompt(prompt)
        .llm(ollama)
        .build()
        .unwrap();

    match chain
        .invoke(prompt_args! { "input" =>
        "Based on this image please find the following informations and give them following precisely this format:
        Event name:
        Location of the event:
        Date and time: (format: HH:DD:MM:YYYY)
        (if the event takes place over multiple days:)
        Days during which the ticket is valid:"
        // "Describe this image"
         })
        .await
    {
        Ok(result) => {
            println!("Result: {}", result);
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    }
}
