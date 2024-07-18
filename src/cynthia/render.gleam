import gleam/regex
import gleam/string

pub fn markdown_to_html(markdown: String) -> String {
  use partialcompiled <- replace(string.trim(markdown), "^  ", "")
  use partialcompiled <- replace(
    partialcompiled,
    "\\*(.*?)\\*",
    "<strong>\\1</strong>",
  )
  use partialcompiled <- replace(partialcompiled, "\\_(.*?)\\_", "<em>\\1</em>")
  use partialcompiled <- replace(
    partialcompiled,
    "\\`(.*?)\\`",
    "<code>\\1</code>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "\\```(.*?)[\\n$]\\`",
    "<code>\\1</code>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "\\[(.*?)\\]\\((.*?)\\)",
    "<a href=\"$2\">\\1</a>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*# (.*?)[\\n$]",
    "<h1>\\1</h1>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*# (.*?)$",
    "<h1>\\1</h1>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*## (.*?)[\\n$]",
    "<h2>\\1</h2>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*## (.*?)$",
    "<h2>\\1</h2>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*### (.*?)[\\n$]",
    "<h3>\\1</h3>",
  )
  use partialcompiled <- replace(partialcompiled, "^### (.*?)$", "<h3>\\1</h3>")
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*#### (.*?)[\\n$]",
    "<h4>\\1</h4>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*#### (.*?)$",
    "<h4>\\1</h4>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*##### (.*?)[\\n$]",
    "<h5>\\1</h5>",
  )
  use partialcompiled <- replace(
    partialcompiled,
    "^\\s*##### (.*?)$",
    "<h5>\\1</h5>",
  )
  use partialcompiled <- replace(partialcompiled, "\n\n", "\n<br>\n")
  let html = partialcompiled
  html
}

fn replace(
  str: String,
  pattern: String,
  replacement: String,
  callback: fn(String) -> String,
) -> String {
  let assert Ok(re) = regex.from_string(pattern)
  callback(regex.replace(each: re, in: str, with: replacement))
}
