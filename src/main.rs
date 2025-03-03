use std::env::args;

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};

#[async_std::main]
async fn main() {
    let path = args().nth(1).expect("no file given");
    let bytes = std::fs::read(path).unwrap();
    let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    println!("PDF content: {}", out);

    let ollama = Ollama::default();
    let model = "deepseek-r1:7b".to_string();
    let prompt = "give me the name and the location of the event, and the duration for which the ticket is valid, keep in minde that the dates are in european format, please write each information on its own line, get the information from this text: ".to_string()
        + out.as_str();

    let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

    if let Ok(res) = res {
        println!("LLM response : {}", res.response);
    }
}
