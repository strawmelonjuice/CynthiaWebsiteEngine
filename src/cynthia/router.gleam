// Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
//
// Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.

import cynthia/web.{type Context}
import gleam/string_builder
import wisp.{type Request, type Response}

pub fn handle_request(req: Request, ctx: Context) -> Response {
  use _req <- web.middleware(req, ctx)
  case wisp.path_segments(req) {
    // This matches `/`.
    //
    [] -> root(req)

    // This matches `/comments`.
    ["comments"] -> root(req)

    // This matches `/comments/:id`.
    // The `id` segment is bound to a variable and passed to the handler.
    // ["comments", id] -> show_comment(req, id)
    // This matches all other paths.
    _ -> wisp.not_found()
  }
}

fn root(_req: Request) {
  let html =
    string_builder.from_string(
      "<!DOCTYPE html>
  <html lang=\"en\">
    <head>
      <meta charset=\"utf-8\">
      <title>Cynthia</title>
      <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
      <link rel=\"stylesheet\" href=\"/static/styles.css\">
    </head>
    <body>
    <h1>Cynthia</h1>
    <p>We'll get there eventually.</p>
      <script src=\"/static/main.js\"></script>
    </body>
  </html>",
    )
  wisp.ok()
  |> wisp.html_body(html)
}
