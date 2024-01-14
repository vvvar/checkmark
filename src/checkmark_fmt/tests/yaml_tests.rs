mod utils;

#[test]
fn yaml() {
    utils::assert_unchanged_after_formatting(
        r#"---
title: "Your document's title"
keywords:
    - A keyword
    - Another keyword
author:
    - Me
---

## Section

And here is a text
"#,
    );
}
