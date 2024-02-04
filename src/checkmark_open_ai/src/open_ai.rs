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

#[derive(Debug, Clone, PartialEq)]
pub struct OpenAiRequestParameters {
    pub role_prompt: String,
    pub user_prompt: String,
    pub response_format: String,
    pub creativity: u8,
    pub api_key: String,
}

impl OpenAiRequestParameters {
    pub fn new(
        role: &str,
        input: &str,
        resp_format: &str,
        creativity: &Option<u8>,
        api_key: &Option<String>,
    ) -> Self {
        Self {
            role_prompt: role.to_string(),
            user_prompt: input.to_string(),
            response_format: resp_format.to_string(),
            creativity: match creativity {
                Some(c) => *c,
                None => 10,
            },
            api_key: match api_key {
                Some(k) => k.clone(),
                None => read_open_ai_api_key().expect("Unable to read Open AI API key"),
            },
        }
    }
}

/// Make a request to the OpenAI.
/// Use ai_role to describe the role that OpenAI assistant shall take.
/// Use user_input as a prompt from user, OpenAI will perform analysis of it.
/// response_format - https://platform.openai.com/docs/api-reference/chat/create#chat-create-response_format
pub async fn open_ai_request(
    params: &OpenAiRequestParameters,
) -> Result<OpenAIResponse, OpenAIError> {
    let requests = params
        .user_prompt
        .split("\n\n#")
        .map(|chunk| {
            let request_data = OpenAIRequestData {
                model: "gpt-3.5-turbo-1106".to_string(),
                n: 1,
                temperature: (params.creativity as f32 / 100.0) * 2.0,
                response_format: OpenAIRequestDataResponseFormat {
                    response_type: params.response_format.clone(),
                },
                messages: vec![
                    OpenAIRequestDataMessage {
                        role: "system".to_string(),
                        content: params.role_prompt.clone(),
                    },
                    OpenAIRequestDataMessage {
                        role: "user".to_string(),
                        content: format!("#{chunk}"),
                    },
                ],
            };
            log::debug!("Sending OpenAI request with data:\n{:#?}", &request_data);
            reqwest::Client::new()
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(&params.api_key)
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
            }
        }
    }
    Ok(open_ai_response)
}
