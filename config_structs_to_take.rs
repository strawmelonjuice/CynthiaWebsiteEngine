/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

pub struct CynthiaConf {
//    pub port: u16,
//    pub cache: Cache,
//    pub pages: Pages,
//    pub generator: Generator,
    pub logging: Logging,
}
//pub struct Cache {
//    pub lifetimes: Lifetimes,
//}

//pub struct Pages {
//    pub notfound_page: String,
//}

//pub struct Lifetimes {
//    pub stylesheets: u64,
//    pub javascript: u64,
//    pub external: u64,
//    pub served: u64,
//}

//pub struct Generator {
//    pub site_baseurl: String,
//    pub og_sitename: String,
//    pub meta: Meta,
//}
//pub struct Meta {
//    pub enable_tags: bool,
//}

pub struct Logging {
    pub(crate) file: FileLogging,
    pub(crate) console: ConsoleLogging,
}

pub struct FileLogging {
    pub filepath: String,
    pub enabled: bool,
    pub cache: bool,
    pub error: bool,
    pub warn: bool,
    pub info: bool,
    pub requests: bool,
    pub proxy_requests: bool,
    pub plugin_asset_requests: bool,
    pub jsr_errors: bool,
}

pub struct ConsoleLogging {
    pub enabled: bool,
    pub cache: bool,
    pub error: bool,
    pub warn: bool,
    pub info: bool,
    pub requests: bool,
    pub proxy_requests: bool,
    pub plugin_asset_requests: bool,
    pub jsr_errors: bool,
}
