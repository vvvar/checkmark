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

/// Get a grammar correction suggestion from the Open AI.
/// NOTE: OPEN_AI_API_KEY env variable shall be set to
/// your API token in order to work
/// Returns a suggestion.
async fn get_open_ai_suggestion(text: &str, context: &Context) -> Result<String, reqwest::Error> {
    let json: serde_json::Value = serde_json::from_str(&format!("
    {{
        \"model\": \"gpt-3.5-turbo\",
        \"messages\": [
            {{
                \"role\": \"system\",
                \"content\": \"You will be provided with statements, and your task is to convert them to standard English.\"
            }},
            {{
                \"role\": \"user\",
                \"content\": \"{}\"
            }}
        ],
        \"temperature\": 0.7,
        \"max_tokens\": 64,
        \"top_p\": 1
    }}", &text)).unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    dotenv::dotenv().ok();
    let api_key = std::env::var("OPEN_AI_API_KEY").unwrap();
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
    // println!("{:#?}", response);

    if let Some(choice) = response.choices.first() {
        if is_false_positive_suggestion(&text, &choice.message.content, &context) {
            return Ok(String::from(text));
        } else {
            return Ok(correct_suggestion(&choice.message.content, &context));
        }
    } else {
        return Ok(String::from(text));
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
                .block_on(get_open_ai_suggestion(&t.value, &context))
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

pub fn check_md_open_ai(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];

    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    analyze_md(&ast, &mut issues, &file, &Context::Document);

    return issues;
}
