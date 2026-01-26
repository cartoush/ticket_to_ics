pub mod openrouter {
    use anyhow::anyhow;
    use openrouter_api::{ChatCompletionRequest, Message, MessageContent, OpenRouterClient, Ready};

    const PROMPT: &str = "Your goal here is to extract the following information: Event name, \
                          Location of the event, Date and time (as UNIX time), Duration (as UNIX time if you find it, leave empty otherwise). \
                          Following this you will find a list of result from an OCR scanner \
                          that has run on a PDF file, it is formatted like so: \
                          \"text: %s\nposition: left: %d top: %d width: %d height: %d\nconfidence: %f\". \
                          Write your response 1 line per field prefixed with the field's name";

    pub async fn openrouter_ocr_result_to_relevant_info(
        client: &OpenRouterClient<Ready>,
        model: &String,
        info: &String,
    ) -> anyhow::Result<String> {
        let txt = PROMPT.to_string() + info.as_str();
        let request = ChatCompletionRequest {
            model: model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::Text(txt),
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
                    return Ok(response.clone());
                }
                MessageContent::Parts(_) => {
                    return Err(anyhow!("Unsupported MessageContent type: Parts"));
                }
            }
        } else {
            return Err(anyhow!("Unhandled ChatCompletionResponse"));
        }
    }
}
