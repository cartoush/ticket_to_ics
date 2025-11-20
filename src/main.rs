use anyhow::anyhow;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use notify::Watcher;
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use openrouter_api::{
    ContentPart, ImageContent, ImageUrl, MessageContent, OpenRouterClient, Ready, TextContent,
    utils,
};
use pdf2image::image::ImageFormat;
use pdf2image::{PDF, RenderOptionsBuilder};
use std::env::{self};
use std::io::{BufWriter, Cursor};
use std::path::Path;
use std::sync::mpsc;

const PROMPT: &str = "Extract the following information from the image: Event name, \
                Location of the event, Date and time. Format these informations \
                as a raw ics file without any formatting, it will be your only output";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = utils::load_api_key_from_env()?;
    let model = env::var("MODEL")?;
    let watchdir = env::var("WATCHDIR")?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new(&watchdir), notify::RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => {
                println!("event: {:?}", event);
                match event.kind {
                    notify::EventKind::Create(_) => {
                        for path in event.paths {
                            let pathstr = path.into_os_string().into_string().unwrap();
                            println!("PATHSTR : {}", pathstr);
                            do_ticket_to_ics(&client, &model, pathstr).await?
                        }
                    }
                    _ => (),
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}

async fn do_ticket_to_ics(
    client: &OpenRouterClient<Ready>,
    model: &String,
    path: String,
) -> anyhow::Result<()> {
    let pdf = PDF::from_file(&path)?;
    let pages = pdf.render(
        pdf2image::Pages::Range(1..=8),
        RenderOptionsBuilder::default().build()?,
    )?;

    let mut writer = BufWriter::new(Cursor::new(Vec::new()));
    pages[0].write_to(&mut writer, ImageFormat::Jpeg)?;

    let buf = writer.into_inner()?.into_inner();
    let image_b64 = "data:image/jpeg;base64,".to_string() + &BASE64_STANDARD.encode(&buf);

    let txt = TextContent {
        content_type: "text".to_string(),
        text: PROMPT.to_string(),
    };
    let img = ImageContent {
        content_type: "image_url".to_string(),
        image_url: ImageUrl {
            url: image_b64,
            detail: None,
        },
    };
    let content_vec = vec![ContentPart::Text(txt), ContentPart::Image(img)];
    let request = ChatCompletionRequest {
        model: model.clone(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::Parts(content_vec),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        stream: Some(false),
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
        tool_choice: None,
        route: None,
        user: None,
        temperature: None,
        max_tokens: None,
        top_p: None,
        top_k: None,
        frequency_penalty: None,
        presence_penalty: None,
        repetition_penalty: None,
        min_p: None,
        top_a: None,
        seed: None,
        stop: None,
        logit_bias: None,
        logprobs: None,
        top_logprobs: None,
        prediction: None,
        parallel_tool_calls: None,
        verbosity: None,
    };

    // Invoke the streaming chat completion endpoint
    let chat_api = client.chat()?;

    let resp = chat_api.chat_completion(request).await?;

    if let Some(choice) = resp.choices.first() {
        match &choice.message.content {
            MessageContent::Text(response) => {
                println!("RESPONSE: {}", response);
                todo!("parse response and push ics file")
            }
            MessageContent::Parts(_) => {
                return Err(anyhow!("Unsupported MessageContent type: Parts"));
            }
        }
    }
    Ok(())
}
