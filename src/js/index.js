globalThis.console = {
  log: (msg) => {
    Deno.core.print(msg);
    return msg;
  }
}

import "/Users/vvoinov/Documents/repos/md-checker/src/js/prettier.js"
import "/Users/vvoinov/Documents/repos/md-checker/src/js/markdown.js"

async function format_markdown(content) {
  return prettier.format(content, { parser: "markdown", plugins: prettierPlugins, });
}

Deno.core.print("initialized");