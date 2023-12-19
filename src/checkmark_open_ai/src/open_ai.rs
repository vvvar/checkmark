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
    pub line_start: usize,
    pub line_end: usize,
    pub problem: String,
    pub fix: String,
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

/// Read OpenAI API key and return it
/// NOTE: OPEN_AI_API_KEY env variable shall be set to
///       your API token in order to work.
///       When no OPEN_AI_API_KEY env var set returns OpenAIError.
fn read_open_ai_api_key() -> Result<String, OpenAIError> {
    dotenv::dotenv().ok();
    if let Ok(api_key) = std::env::var("OPEN_AI_API_KEY") {
        Ok(api_key)
    } else {
        Err(OpenAIError {
            message: "OpenAI API Key is not set. Please set it via OPEN_AI_API_KEY env var"
                .to_string(),
        })
    }
}

/// Make a request to the OpenAI.
/// Use ai_role to describe the role that OpenAI assistant shall take.
/// Use user_input as a prompt from user, OpenAI will perform analysis of it.
/// When all ok - returns a suggestion string.
/// When needed to limit the output - use "max_tokens\": 64
pub async fn open_ai_request(
    ai_role: &str,
    user_input: &str,
) -> Result<OpenAIResponse, OpenAIError> {
    let mut json: serde_json::Value = serde_json::from_str(&format!(
        "
{{
    \"model\": \"gpt-3.5-turbo\",
    \"messages\": [
        {{
            \"role\": \"system\",
            \"content\": \"\"
        }},
        {{
            \"role\": \"user\",
            \"content\": \"\"
        }}
    ],
    \"temperature\": 0.7,
    \"top_p\": 1
}}"
    ))
    .unwrap();
    json["messages"][0]["content"] = serde_json::Value::String(String::from(ai_role));
    json["messages"][1]["content"] = serde_json::Value::String(String::from(user_input));

    let response: OpenAIResponse = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(read_open_ai_api_key()?)
        .json(&json)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    return Ok(response);
}

/// Get a grammar correction suggestion from the Open AI.
pub async fn get_open_ai_grammar_suggestion(text: &str) -> Result<OpenAISuggestion, OpenAIError> {
    let role_prompt = "You will be provided with statements, and your task is to convert them to standard English.";
    let response = open_ai_request(role_prompt, &text).await?;
    if let Some(choice) = response.choices.first() {
        if choice.message.content.ends_with(".") && !text.ends_with(".") {
            return Ok(OpenAISuggestion::Suggestion(String::from(
                choice.message.content.trim_end_matches("."),
            )));
        } else {
            return Ok(OpenAISuggestion::Suggestion(
                choice.message.content.to_string(),
            ));
        }
    } else {
        return Ok(OpenAISuggestion::NoSuggestion);
    }
}

/// Makes a review of provided markdown file with OpenAI
/// Returns string with suggestions
pub async fn get_open_ai_review(file: &common::MarkDownFile) -> Result<OpenAIReview, OpenAIError> {
    let role_prompt = "You will be provided with project documentation in Markdown format.
Your task it to review it.
Ensure it meets high-quality standards.
Provide detailed feedback on grammar, punctuation, sentence structure, formatting, consistency, clarity, readability, and overall coherence.
Additionally, assess the use of active voice, appropriate word choice, and proper citation and referencing.
Aim to enhance the audience perspective, conciseness, and effectiveness of the content.
Do not provide suggestions on Markdown syntax.
Additionally it must contain detailed summary of the review.
Suggestions should describe example which fix could be sufficient.
The resulting must be JSON. It shall have two properties - summary and suggestions.
summary is detailed summary of the review.
suggestions is a list of suggestions that shows what is the problem, where it appear and how to fix that. 
Provide your answer in JSON form. Reply with only the answer in JSON form and include no other commentary:
{
    \"summary\": \"string\",
    \"suggestions\": [
        { \"line_start\": number, \"line_end\": number, \"problem\": \"string\", \"fix\": \"string\" }
    ]
}";
    let response = open_ai_request(role_prompt, &file.content).await?;
    if let Some(choice) = response.choices.first() {
        if let Ok(review) = serde_json::from_str::<OpenAIReview>(&choice.message.content) {
            return Ok(review);
        } else {
            return Ok(OpenAIReview {
                summary: "Everything is ok".to_string(),
                suggestions: vec![],
            });
        }
    } else {
        return Ok(OpenAIReview {
            summary: "Everything is ok".to_string(),
            suggestions: vec![],
        });
    }
}
