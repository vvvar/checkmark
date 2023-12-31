pub enum OpenAISuggestion {
    Suggestion(String),
    NoSuggestion,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAIChoice {
    pub index: i64,
    pub message: OpenAIMessage,
    pub finish_reason: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenAIReviewSuggestion {
    pub description: String,
    pub original: String,
    pub replacement: String,
}

/// Represents review of the Markdown document provided by OpenAI
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenAIReview {
    pub summary: String,
    pub suggestions: Vec<OpenAIReviewSuggestion>,
}

/// Represents an error happened during OpenAI request
#[derive(Debug)]
pub struct OpenAIError {
    pub message: String,
}

impl std::error::Error for OpenAIError {}

impl std::fmt::Display for OpenAIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIRequestDataResponseFormat {
    #[serde(rename = "type")]
    pub response_type: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIRequestDataMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIRequestData {
    pub model: String,
    pub n: usize,
    // pub seed: usize,
    pub temperature: f32,
    pub response_format: OpenAIRequestDataResponseFormat,
    pub messages: Vec<OpenAIRequestDataMessage>,
}

/// Read OpenAI API key and return it
/// NOTE: OPEN_AI_API_KEY env variable shall be set to
///       your API token in order to work.
///       When no OPEN_AI_API_KEY env var set returns OpenAIError.
fn read_open_ai_api_key() -> Result<String, OpenAIError> {
    dotenv::dotenv().ok();
    if let Ok(api_key) = std::env::var("OPEN_AI_API_KEY") {
        Ok(api_key)
    } else {
        log::error!("OPEN_AI_API_KEY env var us not set");
        Err(OpenAIError {
            message: "OpenAI API Key is not set. Please set it via OPEN_AI_API_KEY env var"
                .to_string(),
        })
    }
}

/// Make a request to the OpenAI.
/// Use ai_role to describe the role that OpenAI assistant shall take.
/// Use user_input as a prompt from user, OpenAI will perform analysis of it.
/// response_format - https://platform.openai.com/docs/api-reference/chat/create#chat-create-response_format
pub async fn open_ai_request(
    ai_role: &str,
    user_input: &str,
    response_format: &str,
) -> Result<OpenAIResponse, OpenAIError> {
    let requests = user_input
        .split("\n\n#")
        .map(|chunk| {
            let request_data = OpenAIRequestData {
                model: "gpt-3.5-turbo-1106".to_string(),
                n: 1,
                // seed: 12345,
                temperature: 0.2,
                response_format: OpenAIRequestDataResponseFormat {
                    response_type: response_format.to_string(),
                },
                messages: vec![
                    OpenAIRequestDataMessage {
                        role: "system".to_string(),
                        content: ai_role.to_string(),
                    },
                    OpenAIRequestDataMessage {
                        role: "user".to_string(),
                        content: chunk.to_string(),
                    },
                ],
            };
            log::debug!("Sending OpenAI request with data:\n{:#?}", &request_data);
            reqwest::Client::new()
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(read_open_ai_api_key().unwrap())
                .json(&request_data)
                .send()
        })
        .collect::<Vec<_>>();

    let mut open_ai_response: OpenAIResponse = OpenAIResponse::default();
    for response in futures::future::join_all(requests).await {
        match response {
            Ok(response) => {
                log::debug!("Got response from OpenAI:\n{:#?}\n", &response);
                if response.status() != 200 {
                    log::error!("OpenAI returned error status code: {}", response.status());
                    return Err(OpenAIError {
                        message: "OpenAI returned error status code".to_string(),
                    });
                }
                match &mut response.json::<OpenAIResponse>().await {
                    Ok(response) => {
                        log::debug!(
                            "Successfully parsed response from OpenAI:\n{:#?}",
                            &response
                        );
                        open_ai_response.choices.append(&mut response.choices);
                    }
                    Err(err) => {
                        log::error!("Error parsing response from OpenAI:{:#?}", &err);
                        return Err(OpenAIError {
                            message: "Error parsing response from OpenAI".to_string(),
                        });
                    }
                }
            }
            Err(err) => {
                log::error!("Error sending request to OpenAI:{:#?}", &err);
                // return Err(OpenAIError {
                //     message: "Error sending request to OpenAI".to_string(),
                // });
            }
        }
    }

    Ok(open_ai_response)
}

fn is_false_positive_review_suggestion(suggestion_description: &str) -> bool {
    suggestion_description.contains("a space after the colon")
}

/// Get a grammar correction suggestion from the Open AI.
pub async fn get_open_ai_grammar_suggestion(text: &str) -> Result<OpenAIReview, OpenAIError> {
    let role_prompt = "This is a project documentation written in Markdown split by sections. 
It includes various sections such as Introduction, Installation, Usage, API Reference, and more.
Find all grammatical errors in it.
Ignore text inside code blocks.
The result must be in JSON. It shall have two properties - summary and suggestions.
Suggestions is a list of suggestions that shows where is the problem and provides grammatically correct replacement.
Provide your answer in JSON form. Reply with only the answer in JSON form and include no other commentary:
{
    \"summary\": \"string\",
    \"suggestions\": [
        { \"description\": \"string\", \"original\": \"string\", \"replacement\": \"string\" }
    ]
}";
    return match open_ai_request(role_prompt, text, "json_object").await {
        Ok(response) => {
            match response
                .choices
                .iter()
                .map(|choice| serde_json::from_str::<OpenAIReview>(&choice.message.content))
                .take_while(|e| e.is_ok())
                .map(|e| e.unwrap())
                .reduce(|mut acc, r| {
                    let mut review = OpenAIReview {
                        summary: r.summary,
                        suggestions: vec![],
                    };
                    review.suggestions.append(&mut r.suggestions.clone());
                    review.suggestions.append(&mut acc.suggestions);
                    review
                }) {
                Some(review) => Ok(review),
                None => Err(OpenAIError {
                    message: "OpenAI haven't provided any suggestion".to_string(),
                }),
            }
        }
        Err(err) => Err(err),
    };
}

/// Makes a review of provided markdown file with OpenAI.
/// Returns string with suggestions.
pub async fn get_open_ai_review(
    file: &common::MarkDownFile,
    prompt: &Option<String>,
) -> Result<OpenAIReview, OpenAIError> {
    let response_format_prompt = r#"
Output should be in JSON format with 'summary' and 'suggestions'.
Format your response as follows:
{
    "summary": "<summary of the review>",
    "suggestions": [
        { 
            "description": "<description of the issue>", 
            "original": "<original text>", 
            "replacement": "<suggested fix>" 
        }
    ]
}
Avoid additional commentary.
"#;
    let role_prompt = match &prompt {
        Some(prompt) => format!("{}{}", prompt, response_format_prompt),
        None => format!("{}{}", "
Review this Markdown project documentation for grammar, inconsistencies, inaccuracies, or unclear content.
Ignore style or formatting. Provide a summary and improvement suggestions.
Each suggestion should identify the issue, its location, and a proposed fix.
Each suggestion should have 'description', 'original', and 'replacement'.", response_format_prompt),
    };
    return match open_ai_request(&role_prompt, &file.content, "json_object").await {
        Ok(response) => {
            match response
                .choices
                .iter()
                .map(|choice| serde_json::from_str::<OpenAIReview>(&choice.message.content))
                .take_while(|e| e.is_ok())
                .map(|e| e.unwrap())
                .reduce(|mut acc, r| {
                    let mut review = OpenAIReview {
                        summary: r.summary,
                        suggestions: vec![],
                    };
                    review.suggestions.append(&mut r.suggestions.clone());
                    review.suggestions.append(&mut acc.suggestions);
                    review
                }) {
                Some(review) => Ok(OpenAIReview {
                    summary: review.summary,
                    suggestions: review
                        .suggestions
                        .into_iter()
                        .filter(|suggestion| {
                            !is_false_positive_review_suggestion(&suggestion.description)
                        })
                        .collect(),
                }),
                None => Err(OpenAIError {
                    message: "OpenAI haven't provided any suggestion".to_string(),
                }),
            }
        }
        Err(err) => Err(err),
    };
}
