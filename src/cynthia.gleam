// Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
//
// Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
import argv
import colours
import cynthia/config as cc
import cynthia/router
import cynthia/web.{Context}
import gleam/erlang/process
import gleam/int
import gleam/io
import gleam/option
import gleam/string
import mist
import simplifile
import wisp

pub const version = "1.0.0"

pub fn main() {
  // println!(
  //     "{} - version {}\n by {}{}{} {}!",
  //     "CynthiaEngine".bold().bright_purple(),
  //     env!("CARGO_PKG_VERSION").to_string().green(),
  //     "Straw".bright_red(),
  //     "melon".green(),
  //     "juice".bright_yellow(),
  //     "Mar".magenta()
  // );
  io.println(
    colours.bold(colours.fgdeeppink1("Cynthia"))
    <> " - version "
    <> colours.fggreen(version)
    <> "\nby "
    <> string.replace(
      colours.fgred2("Straw")
        <> colours.fggreen3("melon")
        <> colours.fgyellow1("juice"),
      " ",
      "",
    )
    <> " / "
    <> colours.fgmagenta("MLC Bloeiman"),
  )
  let arguments = argv.load().arguments
  case arguments {
    [] | ["start"] -> go_to_start(option.None)
    ["start", other_directory] -> go_to_start(option.Some(other_directory))
    ["help"] | ["--help"] -> io.println("Cynthia is a web server for Gleam.")
    _ | [_, ..] | [_, _, _, ..] | [_] ->
      io.println(
        "Invalid arguments.
    Run `cynthia --help` for more information.",
      )
  }
}

pub fn go_to_start(other_directory: option.Option(String)) {
  let ld = case other_directory {
    option.Some(s) -> s
    option.None -> {
      let assert Ok(cd) = simplifile.current_directory()
      cd
    }
  }
  io.println("Cynthia is starting in: " <> ld)

  case cc.fetch(ld) {
    Ok(config) -> start(config, ld)
    Error(e) ->
      case string.inspect(e) {
        "UnableToReadFile(Enoent)" ->
          io.println_error(
            "No configuration file found. Please Initialise a CynthiaConfig in the folder you want to start from.

Run `cynthia help` for more information.",
          )
        _ ->
          io.println_error(
            "Failed to load configuration: " <> string.inspect(e),
          )
      }
  }
}

pub fn start(config: cc.CynthiaConf, dir: String) {
  let ctx = Context(active_directory: dir, config: config)
  // This sets the logger to print INFO level logs, and other sensible defaults
  // for a web application.
  wisp.configure_logger()

  // Generate a random secret key base for the application.
  let secret_key_base = wisp.random_string(64)

  // Start the Wisp/Mist web server.
  let handler = router.handle_request(_, ctx)
  case
    wisp.mist_handler(handler, secret_key_base)
    |> mist.new
    |> mist.port(config.port)
    |> mist.after_start(fn(_, _) { io.println("") })
    |> mist.start_http
  {
    Ok(_) ->
      io.println(
        "Server started on http://localhost:" <> int.to_string(config.port),
      )
    Error(e) -> {
      let error_string = string.inspect(e)
      let reason: String = case error_string {
        "SystemError(Eaddrinuse)" ->
          ": The port is already in use. Try stopping the other server or changing the port. [SystemError(Eaddrinuse)]"
        _ -> " [" <> error_string <> "]"
      }
      io.println_error("Failed to start server" <> reason)
    }
  }
  // Put current process to sleep forever, as the server is now running.
  process.sleep_forever()
}
