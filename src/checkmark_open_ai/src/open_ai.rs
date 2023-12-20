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

/// Check is grammar suggestion is false-positive
fn is_false_positive_suggestion(suggestion: &str, original: &str) -> bool {
    if suggestion.is_empty() {
        // No suggestion
        true
    } else if original.len() < 3 {
        // Less then 3 symbols is most likely is not a sentence/statement
        true
    } else if suggestion
        .to_lowercase()
        .contains("unable to provide any corrections or feedback without any context or user input")
    {
        true
    } else if suggestion
        .to_lowercase()
        .contains("you haven't provided any statement for me to convert to standard English")
    {
        true
    } else if suggestion.to_ascii_lowercase().contains("sounds good") {
        true
    } else if suggestion
        .to_lowercase()
        .contains("is not a statement that can be converted to standard English")
    {
        true
    } else if suggestion
        .to_lowercase()
        .trim_end_matches(".")
        .eq(&original.to_lowercase())
    {
        true
    } else if suggestion.to_lowercase().eq(&original.to_lowercase()) {
        true
    } else {
        false
    }
}

/// Sometimes we want to adjust suggestion a bit, this function does that.
/// For example, OpenAI may suggest adding period when it si not needed.
fn auto_correct_grammar_suggestion(suggestion: &str, original: &str) -> String {
    if suggestion.ends_with(".") && !original.ends_with(".") {
        return suggestion.strip_suffix(".").unwrap().to_string();
    } else {
        return suggestion.to_string();
    }
}

fn is_false_positive_review_suggestion(suggestion_description: &str) -> bool {
    if suggestion_description.contains("a space after the colon") {
        return true;
    } else {
        return false;
    }
}

/// Get a grammar correction suggestion from the Open AI.
pub async fn get_open_ai_grammar_suggestion(text: &str) -> Result<OpenAISuggestion, OpenAIError> {
    let role_prompt = "You are a grammar checker that.
You take all the users input and auto correct it.
Just reply to user input with the correct grammar.
DO NOT reply the context of the question of the user input.
If the user input is grammatically correct then just reply “sounds good” and nothing else.";
    let response = open_ai_request(role_prompt, &text).await?;
    if let Some(choice) = response.choices.first() {
        if is_false_positive_suggestion(&choice.message.content, &text) {
            return Ok(OpenAISuggestion::NoSuggestion);
        } else {
            return Ok(OpenAISuggestion::Suggestion(
                auto_correct_grammar_suggestion(&choice.message.content, &text),
            ));
        }
    } else {
        return Ok(OpenAISuggestion::NoSuggestion);
    }
}

/// Makes a review of provided markdown file with OpenAI.
/// Returns string with suggestions.
pub async fn get_open_ai_review(file: &common::MarkDownFile) -> Result<OpenAIReview, OpenAIError> {
    let role_prompt = "You will be provided with project documentation in Markdown format.
Your task it to review it and provide suggestions for improvement.
Ensure it meets high-quality standards.
Provide detailed feedback on grammar, punctuation, sentence structure, formatting, consistency, clarity, readability, and overall coherence.
Additionally, assess the use of active voice, appropriate word choice, and proper citation and referencing.
Aim to enhance the audience perspective, conciseness, and effectiveness of the content.
DO NOT provide suggestions on Markdown syntax.
Additionally it must contain detailed summary of the review.
Suggestions should describe example which fix could be sufficient.
The resulting must be JSON. It shall have two properties - summary and suggestions.
Suggestions is a list of suggestions that shows what is the problem, where it appear(exact row numbers in the original document where problem appears) and how to fix that.
Provide your answer in JSON form. Reply with only the answer in JSON form and include no other commentary:
{
    \"summary\": \"string\",
    \"suggestions\": [
        { \"description\": \"string\", \"original\": \"string\", \"replacement\": \"string\" }
    ]
}
";
    let response = open_ai_request(role_prompt, &file.content).await?;
    if let Some(choice) = response.choices.first() {
        if let Ok(review) = serde_json::from_str::<OpenAIReview>(&choice.message.content) {
            return Ok(OpenAIReview {
                summary: review.summary,
                suggestions: review.suggestions
                                .into_iter()
                                .filter(|suggestion| !is_false_positive_review_suggestion(&suggestion.description))
                                .collect(),
            });
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
