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
use ics::{self, components::Property, ICalendar};

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
        "Based on this picture of a concert ticket please find the following informations and give them following precisely this format:
        Event name: <name>
        Location of the event: <location>
        Date and time: <format: HH:DD:MM:YYYY>
        (if the event takes place over multiple days:)
        Days during which the ticket is valid: <DD:MM to DD:MM>"
        // "Describe this image"
         })
        .await
    {
        Ok(result) => {
            println!("Result: {}", result);
            
            // Parse the result to extract event details
            let mut event_name = String::new();
            let mut location = String::new();
            let mut date_time = String::new();
            let mut end_date = String::new();
            
            for line in result.lines() {
                if line.starts_with("Event name:") {
                    event_name = line.replace("Event name:", "").trim().to_string();
                } else if line.starts_with("Location of the event:") {
                    location = line.replace("Location of the event:", "").trim().to_string();
                } else if line.starts_with("Date and time:") {
                    date_time = line.replace("Date and time:", "").trim().to_string();
                } else if line.starts_with("Days during which the ticket is valid:") {
                    let dates = line.replace("Days during which the ticket is valid:", "");
                    let dates = dates.trim();
                    if let Some((start, end)) = dates.split_once(" to ") {
                        date_time = start.trim().to_string();
                        end_date = end.trim().to_string();
                    }
                }
            }

            // Create ICS 
            let mut calendar = ICalendar::new("2.0", "-//Ticket to ICS//EN");
            let mut event = ics::Event::new(
                format!("event_{}", chrono::Utc::now().timestamp()),
                chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
            );

            event.push(Property::new("SUMMARY", &event_name));
            event.push(Property::new("LOCATION", location));
            
            // Parse and format the date/time
            if !date_time.is_empty() {
                let dt = chrono::NaiveDateTime::parse_from_str(&date_time, "%H:%d:%m:%Y")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc());
                event.push(Property::new(
                    "DTSTART",
                    dt.format("%Y%m%dT%H%M%S").to_string(),
                ));
                
                if !end_date.is_empty() {
                    let end_dt = chrono::NaiveDate::parse_from_str(&end_date, "%d:%m")
                        .unwrap_or_else(|_| chrono::Utc::now().date_naive())
                        .and_hms_opt(23, 59, 59)
                        .unwrap();
                    event.push(Property::new(
                        "DTEND",
                        end_dt.format("%Y%m%dT%H%M%S").to_string(),
                    ));
                } else {
                    // Default to 2 hours duration if no end time specified
                    let end_dt = dt + chrono::Duration::hours(2);
                    event.push(Property::new(
                        "DTEND",
                        end_dt.format("%Y%m%dT%H%M%S").to_string(),
                    ));
                }
            }

            calendar.add_event(event);
            
            // Save the ICS file
            let ics_path = format!("{}.ics", event_name.replace(" ", "_"));
            std::fs::write(&ics_path, calendar.to_string()).expect("Failed to write ICS file");
            println!("ICS file created: {}", ics_path);
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    };

    // Now let's turn result into an .ics file
    
}
