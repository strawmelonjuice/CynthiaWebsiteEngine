// Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
//
// Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.

import cynthia/config.{type CynthiaConf}
import simplifile
import wisp

pub type Context {
  Context(active_directory: String, config: CynthiaConf)
}

pub fn middleware(
  req: wisp.Request,
  ctx: Context,
  handle_request: fn(wisp.Request) -> wisp.Response,
) -> wisp.Response {
  let req = wisp.method_override(req)
  use <- wisp.log_request(req)
  use <- wisp.rescue_crashes
  use req <- wisp.handle_head(req)
  let assetsdir = ctx.active_directory <> "/assets"

  case simplifile.is_directory(assetsdir) {
    Ok(True) -> {
      use <- wisp.serve_static(req, under: "/assets", from: assetsdir)
      handle_request(req)
    }
    Ok(False) -> {
      handle_request(req)
    }
    Error(_) -> {
      handle_request(req)
    }
  }

  handle_request(req)
}
