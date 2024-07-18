import cynthia/render
import gleam/io
import gleeunit
import gleeunit/should

pub fn main() {
  gleeunit.main()
}

// gleeunit test functions end in `_test`
pub fn markdown_render_test() {
  let markdown =
    "
  # Hello
  _This_ is a test *for markdown*.
  ~~This is a test for strikethrough~~.
  ### This is a list
  - This is a list item
  - This is another list item
  "
  io.println(
    markdown
    <> " in markdown is `"
    <> render.markdown_to_html("# Hello")
    <> "` in HTML",
  )
  1
  |> should.equal(1)
}
