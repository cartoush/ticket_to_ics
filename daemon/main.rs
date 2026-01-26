use ocr_rs::OcrEngine;
use openrouter_api::{OpenRouterClient, utils};
use std::env::{self};

use crate::{
    ocr::ocr::{ocr_do, ocr_to_string},
    openrouter::openrouter::openrouter_ocr_result_to_relevant_info,
};

mod ocr;
mod openrouter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = utils::load_api_key_from_env()?;
    let model = env::var("MODEL")?;
    // let watchdir = env::var("WATCHDIR")?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    let engine = OcrEngine::new(
        "./models/PP-OCRv5_mobile_det.mnn",
        "./models/latin_PP-OCRv5_mobile_rec_infer.mnn",
        "./models/ppocr_keys_latin.txt",
        None,
    )?;

    // let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    // let mut watcher = notify::recommended_watcher(tx)?;
    // watcher.watch(Path::new(&watchdir), notify::RecursiveMode::Recursive)?;

    // for res in rx {
    //     match res {
    //         Ok(event) => {
    //             println!("event: {:?}", event);
    //             match event.kind {
    //                 notify::EventKind::Create(_) => {
    //                     for path in event.paths {
    //                         let pathstr = path.into_os_string().into_string().unwrap();
    //                         println!("PATHSTR : {}", pathstr);

    //                         // do_ticket_to_ics(&client, &model, pathstr).await?
    //                     }
    //                 }
    //                 _ => (),
    //             }
    //         }
    //         Err(e) => println!("watch error: {:?}", e),
    //     }
    // }
    // Ok(())
    //
    let ocr_res = ocr_do(
        &engine,
        &"/home/artis/Downloads/Aymericlompret.pdf".to_string(),
    )?;
    let lines = ocr_to_string(ocr_res)?;
    openrouter_ocr_result_to_relevant_info(&client, &model, &lines).await?;
    Ok(())
}
