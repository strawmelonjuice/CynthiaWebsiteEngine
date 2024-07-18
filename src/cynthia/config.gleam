// Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
//
// Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.

// import gleam/dynamic.{field, int, list, string}
import gleam/bool
import gleam/int
import gleam/io
import gleam/result
import simplifile
import tom

/// Fetches the configuration file from the given directory and returns a `CynthiaConf` struct.
pub fn fetch(ld: String) -> Result(CynthiaConf, CynthiaConfigLoadErrors) {
  let fulldefaultconfig: CynthiaConf =
    CynthiaConf(
      port: 3000,
      cache: CynthiaConfCache(lifetimes: CynthiaConfCacheLifetimes(
        stylesheets: 72_000,
        javascript: 1200,
        forwarded: 1600,
        served: 50,
      )),
      pages: CynthiaConfPages(missing: "404"),
      generated: CynthiaConfGenerated(
        site_baseurl: "",
        og_sitename: "My Cynthia Site!",
        meta: CynthiaConfGeneratedMeta(enable_tags: False),
      ),
    )

  use toml_str <- result.try(
    simplifile.read(ld <> "/cynthia.toml")
    |> result.map_error(UnableToReadFile),
  )
  // io.println("Config file contents: " <> toml_str)

  use parsed_toml <- result.try(
    tom.parse(toml_str)
    |> result.map_error(ParseTomlError),
  )
  // io.debug(parsed_toml)
  let constructed_config: CynthiaConf =
    CynthiaConf(
      port: result.lazy_unwrap(tom.get_int(parsed_toml, ["port"]), fn() {
        result.lazy_unwrap(tom.get_int(parsed_toml, ["PORT"]), fn() {
          io.println_error(
            "Port not found in config file! Defaulting to "
            <> int.to_string(fulldefaultconfig.port),
          )
          fulldefaultconfig.port
        })
      }),
      cache: CynthiaConfCache(lifetimes: CynthiaConfCacheLifetimes(
        stylesheets: result.lazy_unwrap(
          tom.get_int(parsed_toml, ["cache", "lifetimes", "stylesheets"]),
          fn() {
            io.println_error(
              "Stylesheets cache lifetime not found in config file! Defaulting to \""
              <> int.to_string(fulldefaultconfig.cache.lifetimes.stylesheets)
              <> "\"",
            )
            fulldefaultconfig.cache.lifetimes.stylesheets
          },
        ),
        javascript: result.lazy_unwrap(
          tom.get_int(parsed_toml, ["cache", "lifetimes", "javascript"]),
          fn() {
            io.println_error(
              "Javascript cache lifetime not found in config file! Defaulting to "
              <> int.to_string(fulldefaultconfig.cache.lifetimes.javascript),
            )
            fulldefaultconfig.cache.lifetimes.javascript
          },
        ),
        forwarded: result.lazy_unwrap(
          tom.get_int(parsed_toml, ["cache", "lifetimes", "forwarded"]),
          fn() {
            io.println_error(
              "Forwarded cache lifetime not found in config file! Defaulting to "
              <> int.to_string(fulldefaultconfig.cache.lifetimes.forwarded),
            )
            fulldefaultconfig.cache.lifetimes.forwarded
          },
        ),
        served: result.lazy_unwrap(
          tom.get_int(parsed_toml, ["cache", "lifetimes", "served"]),
          fn() {
            io.println_error(
              "Served cache lifetime not found in config file! Defaulting to "
              <> int.to_string(fulldefaultconfig.cache.lifetimes.served),
            )
            fulldefaultconfig.cache.lifetimes.served
          },
        ),
      )),
      pages: CynthiaConfPages(
        missing: result.lazy_unwrap(
          tom.get_string(parsed_toml, ["pages", "missing"]),
          fn() { fulldefaultconfig.pages.missing },
        ),
      ),
      generated: CynthiaConfGenerated(
        site_baseurl: result.lazy_unwrap(
          tom.get_string(parsed_toml, ["generator", "site_baseurl"]),
          fn() { "" },
        ),
        og_sitename: result.lazy_unwrap(
          tom.get_string(parsed_toml, ["generator", "og_sitename"]),
          fn() {
            result.lazy_unwrap(
              tom.get_string(parsed_toml, ["generator", "og-sitename"]),
              fn() {
                result.lazy_unwrap(
                  tom.get_string(parsed_toml, ["generator", "og-site-name"]),
                  fn() {
                    io.println_error(
                      "og-site-name not found in config file! Defaulting to "
                      <> fulldefaultconfig.generated.og_sitename,
                    )
                    fulldefaultconfig.generated.og_sitename
                  },
                )
              },
            )
          },
        ),
        meta: CynthiaConfGeneratedMeta(
          enable_tags: result.lazy_unwrap(
            tom.get_bool(parsed_toml, ["generator", "meta", "enable_tags"]),
            fn() {
              result.lazy_unwrap(
                tom.get_bool(parsed_toml, ["generator", "meta", "enable-tags"]),
                fn() {
                  io.println_error(
                    "enable-tags not found in config file! Defaulting to "
                    <> bool.to_string(
                      fulldefaultconfig.generated.meta.enable_tags,
                    ),
                  )
                  fulldefaultconfig.generated.meta.enable_tags
                },
              )
            },
          ),
        ),
      ),
    )
  // io.debug(constructed_config)
  Ok(constructed_config)
}

pub type CynthiaConf {
  CynthiaConf(
    port: Int,
    cache: CynthiaConfCache,
    pages: CynthiaConfPages,
    generated: CynthiaConfGenerated,
  )
}

pub type CynthiaConfCache {
  CynthiaConfCache(lifetimes: CynthiaConfCacheLifetimes)
}

pub type CynthiaConfCacheLifetimes {
  CynthiaConfCacheLifetimes(
    stylesheets: Int,
    javascript: Int,
    forwarded: Int,
    served: Int,
  )
}

pub type CynthiaConfPages {
  CynthiaConfPages(missing: String)
}

pub type CynthiaConfGenerated {
  CynthiaConfGenerated(
    site_baseurl: String,
    og_sitename: String,
    meta: CynthiaConfGeneratedMeta,
  )
}

pub type CynthiaConfGeneratedMeta {
  CynthiaConfGeneratedMeta(enable_tags: Bool)
}

pub type CynthiaConfigLoadErrors {
  CannotFindCWDError(simplifile.FileError)
  UnknownError
  UnknownElaboratedError(String)
  UnableToReadFile(simplifile.FileError)
  ParseTomlError(tom.ParseError)
  MissingNameError(tom.GetError)
  EmptyNameError
  ValueMissingError(tom.GetError)
}
