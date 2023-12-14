#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIChoice {
    pub index: i64,
    pub message: OpenAIMessage,
    pub finish_reason: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIReviewSuggestion {
    line_start: usize,
    line_end: usize,
    problem: String,
    fix: String,
}

/// Represents review of the Markdown document provided by OpenAI
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIReview {
    summary: String,
    suggestions: Vec<OpenAIReviewSuggestion>,
}

/// Sometimes OpenAI provides false-positives
/// such as suggestion to append period(".") symbol
/// for text. This function checks for such cases
fn is_false_positive_suggestion(original: &str, suggestion: &str, context: &Context) -> bool {
    if let Context::Heading = context {
        // When the only suggestion was just to append period(".")
        // to the end of the heading then it is undesirable
        if suggestion.len() == original.len() + 1 && suggestion.ends_with(".") {
            return true;
        }
    }
    return false;
}

/// For some cases we want to slightly adjust our suggestion.
/// For example, when providing a suggestion for the heading with
/// period(".") symbol at the end we want to strip it away because
/// periods in headings are forbidden
fn correct_suggestion(suggestion: &str, context: &Context) -> String {
    if let Context::Heading = context {
        if suggestion.ends_with(".") {
            return String::from(suggestion.trim_end_matches("."));
        }
    }
    return String::from(suggestion);
}

/// Make a request to the OpenAI.
/// Use role_description to describe the role that OpenAI assistant shall take.
/// Use user_input as a prompt from user, OpenAI will perform analysis of it.
/// NOTE: OPEN_AI_API_KEY env variable shall be set to
///       your API token in order to work.
///       When no OPEN_AI_API_KEY env var set returns empty response.
/// When all ok - returns a suggestion string.
async fn open_ai_request(
    role_description: &str,
    user_input: &str,
) -> Result<String, reqwest::Error> {
    dotenv::dotenv().ok();
    // When needed to limit the output - use "max_tokens\": 64
    if let Ok(api_key) = std::env::var("OPEN_AI_API_KEY") {
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

        json["messages"][0]["content"] = serde_json::Value::String(String::from(role_description));
        json["messages"][1]["content"] = serde_json::Value::String(String::from(user_input));

        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
        );

        let response: OpenAIResponse = reqwest::Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .headers(headers)
            .json(&json)
            .send()
            .await?
            .json()
            .await?;

        if let Some(choice) = response.choices.first() {
            return Ok(String::from(&choice.message.content));
        } else {
            return Ok(String::from(""));
        }
    } else {
        return Ok(String::from(""));
    }
}

/// Get a grammar correction suggestion from the Open AI.
async fn get_open_ai_grammar_suggestion(
    text: &str,
    context: &Context,
) -> Result<String, reqwest::Error> {
    if let Ok(suggestion) = open_ai_request("You will be provided with statements, and your task is to convert them to standard English.", &text).await {
        if is_false_positive_suggestion(&text, &suggestion, &context) {
            return Ok(String::from(text));
        } else {
            return Ok(correct_suggestion(&suggestion, &context));
        }
    } else {
        return Ok(String::from(text));
    }
}

/// Makes a review of provided markdown file with OpenAI
/// Returns string with suggestions
async fn get_open_ai_review(file: &common::MarkDownFile) -> Result<OpenAIReview, reqwest::Error> {
    if let Ok(suggestion) = open_ai_request(
"You will be provided with project documentation in Markdown format.
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
}
", &file.content).await {
        if let Ok(review) = serde_json::from_str::<OpenAIReview>(&suggestion) {
            return Ok(review);
        } else {
            return Ok(OpenAIReview {
                summary: "Everything is ok".to_string(),
                suggestions: vec![]
            });
        }
    } else {
        return Ok(OpenAIReview {
            summary: "Everything is ok".to_string(),
            suggestions: vec![]
        });
    }
}

/// Current parsing context
enum Context {
    Document,
    Heading,
}

/// Analyze provided markdown file(in form of AST)
/// and fill issues when found
fn analyze_md(
    node: &markdown::mdast::Node,
    mut issues: &mut Vec<common::CheckIssue>,
    file: &common::MarkDownFile,
    context: &Context,
) {
    match node {
        markdown::mdast::Node::Heading(heading) => {
            for child in &heading.children {
                analyze_md(&child, &mut issues, &file, &Context::Heading);
            }
        }
        markdown::mdast::Node::Text(t) => {
            if let Ok(suggestion) = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(get_open_ai_grammar_suggestion(&t.value, &context))
            {
                if !suggestion.eq(&t.value) {
                    let mut row_num_start = 0;
                    let mut row_num_end = 0;
                    let mut col_num_start = 0;
                    let mut col_num_end = 0;
                    if let Some(position) = &t.position {
                        row_num_start = position.start.line;
                        row_num_end = position.end.line;
                        col_num_start = position.start.column;
                        col_num_end = position.end.column;
                    }

                    issues.push(
                        common::CheckIssueBuilder::default()
                            .set_category(common::IssueCategory::Grammar)
                            .set_file_path(file.path.clone())
                            .set_row_num_start(row_num_start)
                            .set_row_num_end(row_num_end)
                            .set_col_num_start(col_num_start)
                            .set_col_num_end(col_num_end)
                            .set_message(String::from("Consider provided grammar suggestions"))
                            .set_fixes(vec![suggestion])
                            .build(),
                    );
                }
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    analyze_md(&child, &mut issues, &file, &Context::Document);
                }
            }
        }
    }
}

pub fn check_grammar(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];

    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    analyze_md(&ast, &mut issues, &file, &Context::Document);

    return issues;
}

pub fn make_a_review(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];

    if let Ok(review) = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_open_ai_review(&file))
    {
        if !review.suggestions.is_empty() {
            issues.push(
                common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::Review)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(0)
                    .set_row_num_end(0)
                    .set_col_num_start(0)
                    .set_col_num_end(0)
                    .set_message(String::from(&review.summary))
                    .build(),
            );
            for suggestion in &review.suggestions {
                issues.push(
                    common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Review)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(suggestion.line_start)
                        .set_row_num_end(suggestion.line_end)
                        .set_col_num_start(0)
                        .set_col_num_end(0)
                        .set_message(suggestion.problem.clone())
                        .push_fix(&suggestion.fix.clone())
                        .build(),
                );
            }
        }
    }

    return issues;
}
