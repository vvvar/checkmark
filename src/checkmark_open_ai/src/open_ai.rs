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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAIReviewSuggestion {
    pub description: String,
    pub original: String,
    pub replacement: String,
}

/// Represents review of the Markdown document provided by OpenAI
#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
    pub seed: usize,
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
pub async fn open_ai_request(
    ai_role: &str,
    user_input: &str,
) -> Result<OpenAIResponse, OpenAIError> {
    let request_data = OpenAIRequestData {
        model: "gpt-3.5-turbo-1106".to_string(),
        n: 1,
        seed: 12345,
        temperature: 0.1,
        response_format: OpenAIRequestDataResponseFormat {
            response_type: "json_object".to_string(),
        },
        messages: vec![
            OpenAIRequestDataMessage {
                role: "system".to_string(),
                content: ai_role.to_string(),
            },
            OpenAIRequestDataMessage {
                role: "user".to_string(),
                content: user_input.to_string(),
            },
        ],
    };
    log::debug!("Sending OpenAI request with data:\n{:#?}", &request_data);
    match reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(read_open_ai_api_key()?)
        .json(&request_data)
        .send()
        .await
    {
        Ok(response) => {
            log::debug!("HTTP response from OpenAI:\n{:#?}", &response);
            let open_ai_response: OpenAIResponse = response.json().await.unwrap();
            log::debug!("Response body from OpenAI:\n{:#?}", &open_ai_response);
            Ok(open_ai_response)
        }
        Err(err) => {
            log::error!("Error from OpenAI:\n{:#?}", &err);
            Err(OpenAIError {
                message: "Error sending request to the OpenAI".to_string(),
            })
        }
    }
}

fn is_false_positive_review_suggestion(suggestion_description: &str) -> bool {
    suggestion_description.contains("a space after the colon")
}

/// Get a grammar correction suggestion from the Open AI.
pub async fn get_open_ai_grammar_suggestion(text: &str) -> Result<OpenAIReview, OpenAIError> {
    let role_prompt = "This is a Markdown document.
Please check it for any grammatical errors and suggest corrections.
Ignore any other potential issues like style or formatting.
The result must be in JSON. It shall have two properties - summary and suggestions.
Suggestions is a list of suggestions that shows what is the problem, where it appear and how to fix that.
Provide your answer in JSON form. Reply with only the answer in JSON form and include no other commentary:
{
    \"summary\": \"string\",
    \"suggestions\": [
        { \"description\": \"string\", \"original\": \"string\", \"replacement\": \"string\" }
    ]
}";
    return match open_ai_request(role_prompt, text).await {
        Ok(response) => match response.choices.first() {
            Some(choice) => match serde_json::from_str::<OpenAIReview>(&choice.message.content) {
                Ok(review) => {
                    log::debug!("Got a grammar review from OpenAI:\n{:#?}", &review);
                    Ok(review)
                }
                Err(err) => {
                    log::error!("Error parsing response from OpenAI:{:#?}", &err);
                    Err(OpenAIError {
                        message: "Error parsing response from OpenAI".to_string(),
                    })
                }
            },
            None => {
                log::error!("OpenAI replied without suggestions:\n{:#?}", &response);
                Err(OpenAIError {
                    message: "OpenAI replied without suggestions".to_string(),
                })
            }
        },
        Err(err) => Err(err),
    };
}

/// Makes a review of provided markdown file with OpenAI.
/// Returns string with suggestions.
pub async fn get_open_ai_review(file: &common::MarkDownFile) -> Result<OpenAIReview, OpenAIError> {
    let role_prompt = "You will be provided with project documentation in Markdown format.
Your task it to review a text in it and provide suggestions for improvement. Ensure it meets high-quality standards.
Provide detailed feedback on grammar, punctuation, sentence structure, consistency, clarity, readability, and overall coherence.
Additionally, assess the use of active voice, appropriate word choice, and proper citation and referencing.
Aim to enhance the audience perspective, conciseness, and effectiveness of the content.
Additionally it must contain detailed summary of the review.
Suggestions should describe example which fix could be sufficient.
Replacement should provide example how this can be fixed.
The resulting must be JSON. It shall have two properties - summary and suggestions.
Suggestions is a list of suggestions that shows what is the problem, where it appear and how to fix that.
Provide your answer in JSON form. Reply with only the answer in JSON form and include no other commentary:
{
    \"summary\": \"string\",
    \"suggestions\": [
        { \"description\": \"string\", \"original\": \"string\", \"replacement\": \"string\" }
    ]
}
";
    return match open_ai_request(role_prompt, &file.content).await {
        Ok(response) => match response.choices.first() {
            Some(choice) => match serde_json::from_str::<OpenAIReview>(&choice.message.content) {
                Ok(review) => {
                    log::debug!("Got a review from OpenAI:\n{:#?}", &review);
                    Ok(OpenAIReview {
                        summary: review.summary,
                        suggestions: review
                            .suggestions
                            .into_iter()
                            .filter(|suggestion| {
                                !is_false_positive_review_suggestion(&suggestion.description)
                            })
                            .collect(),
                    })
                }
                Err(err) => {
                    log::error!("Error parsing response from OpenAI:{:#?}", &err);
                    Err(OpenAIError {
                        message: "Error parsing response from OpenAI".to_string(),
                    })
                }
            },
            None => {
                log::error!("OpenAI replied without suggestions:\n{:#?}", &response);
                Err(OpenAIError {
                    message: "OpenAI replied without suggestions".to_string(),
                })
            }
        },
        Err(err) => Err(err),
    };
}
