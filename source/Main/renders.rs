/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use actix_web::web::Data;
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use crate::config::CynthiaConfClone;
use crate::publications::{
    Author, CynthiaPublicationDates, CynthiaPublicationList, CynthiaPublicationListTrait,
};
use crate::{LockCallback, ServerContext};
use colored::Colorize;

pub(crate) enum PGIDCheckResponse {
    Ok,
    Error,
    NotFound,
}

#[derive(Clone)]
pub(crate) enum RenderrerResponse {
    Error,
    NotFound,
    Ok(String),
}
#[allow(unused)]
impl RenderrerResponse {
    /// Returns true if the GenerationResponse is ok.
    pub fn is_ok(&self) -> bool {
        matches!(self, RenderrerResponse::Ok(_))
    }
    /// Returns true if the GenerationResponse is not found.
    pub fn is_not_found(&self) -> bool {
        matches!(self, RenderrerResponse::NotFound)
    }
    /// Returns true if the GenerationResponse is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, RenderrerResponse::Error)
    }
    /// Unwraps the GenerationResponse into a String.
    pub fn unwrap(self) -> String {
        match self {
            RenderrerResponse::Ok(s) => s,
            _ => String::new(),
        }
    }
    fn within(&mut self, then: fn(inner: String) -> String) -> &Self {
        let ob = self.clone();
        if matches!(self, RenderrerResponse::Ok(_)) {
            let inner = ob.unwrap();
            let new_inner = then(inner);
            *self = RenderrerResponse::Ok(new_inner);
        }
        self
    }
}

pub(crate) fn check_pgid(
    pgid: String,
    server_context: &MutexGuard<ServerContext>,
) -> PGIDCheckResponse {
    let page_id = if pgid == *"" {
        String::from("root")
    } else {
        pgid
    };
    let published = CynthiaPublicationList::load();

    if !published.validate(server_context.config.clone()) {
        error!("Incorrect publications found in publications.jsonc.");
        return PGIDCheckResponse::Error;
    }
    let publication = published.get_by_id(page_id);
    if publication.is_none() {
        let publication = published.get_by_id(server_context.config.pages.notfound_page.clone());
        if publication.is_none() {
            error!(
                "No 404 page found in publications.jsonc, or incorrectly defined in CynthiaConfig."
            );
            PGIDCheckResponse::Error
        } else {
            PGIDCheckResponse::NotFound
        }
    } else {
        PGIDCheckResponse::Ok
    }
}
pub(crate) async fn render_from_pgid(
    pgid: String,
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
) -> RenderrerResponse {
    let config = server_context_mutex
        .lock_callback(|a| a.config.clone())
        .await;
    let published = CynthiaPublicationList::load();
    let publication = if pgid == *"" {
        published.get_root()
    } else {
        published.get_by_id(pgid)
    };
    if publication.is_none() {
        if published.get_notfound(config).is_none() {
            RenderrerResponse::Error
        } else {
            RenderrerResponse::NotFound
        }
    } else if let Some(pb) = publication {
        in_renderer::render_controller(pb, server_context_mutex.clone()).await
    } else {
        RenderrerResponse::Error
    }
}

/// This struct is a stripped down version of the Scene struct in the config module.
/// It stores only the necessary data for rendering a single publication.
struct PublicationScene {
    template: String,
    stylesheet: Option<String>,
    script: Option<String>,
    kind: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PageLikePublicationTemplateData {
    meta: PageLikePublicationTemplateDataMeta,
    content: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PageLikePublicationTemplateDataMeta {
    id: String,
    title: String,
    desc: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    author: Option<Author>,
    dates: CynthiaPublicationDates,
    thumbnail: Option<String>,
}

mod in_renderer {
    use super::*;
    use crate::externalpluginservers::EPSRequestBody;
    use crate::{
        config::{CynthiaConfig, Scene, SceneCollectionTrait},
        publications::{ContentType, CynthiaPublication, PublicationContent},
    };
    use handlebars::{handlebars_helper, Handlebars};
    use log::warn;
    use std::path::PathBuf;
    use std::{fs, path::Path};
    use ContentType::Html;

    pub(super) async fn render_controller(
        publication: CynthiaPublication,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> RenderrerResponse {
        let config = server_context_mutex
            .lock_callback(|a| a.config.clone())
            .await;
        let scene = fetch_scene(publication.clone(), config.clone());

        if scene.is_none() {
            error!("No scene found for publication.");
            return RenderrerResponse::Error;
        };
        let scene = scene.unwrap();
        let localscene = match publication {
            CynthiaPublication::Page { .. } => PublicationScene {
                template: scene.templates.page.clone(),
                stylesheet: scene.stylefile.clone(),
                script: scene.script.clone(),
                kind: "page".to_string(),
            },
            CynthiaPublication::Post { .. } => PublicationScene {
                template: scene.templates.post.clone(),
                stylesheet: scene.stylefile.clone(),
                script: scene.script.clone(),
                kind: "post".to_string(),
            },
            CynthiaPublication::PostList { .. } => PublicationScene {
                template: scene.templates.postlist.clone(),
                stylesheet: scene.stylefile.clone(),
                script: scene.script.clone(),
                kind: "postlist".to_string(),
            },
        };

        let pageish_template_data: PageLikePublicationTemplateData = match publication {
            CynthiaPublication::Page {
                pagecontent,
                id,
                title,
                thumbnail,
                description,
                dates,
                ..
            } => PageLikePublicationTemplateData {
                meta: PageLikePublicationTemplateDataMeta {
                    id: id.clone(),
                    title: title.clone(),
                    desc: description.clone(),
                    category: None,
                    author: None,
                    tags: vec![],
                    dates: dates.clone(),
                    thumbnail: thumbnail.clone(),
                },
                content: match fetch_page_ish_content(pagecontent).await.unwrap_html() {
                    RenderrerResponse::Ok(s) => s,
                    _ => return RenderrerResponse::Error,
                },
            },
            CynthiaPublication::Post {
                id,
                title,
                short,
                dates,
                thumbnail,
                category,
                author,
                postcontent,
                tags,
                ..
            } => PageLikePublicationTemplateData {
                meta: PageLikePublicationTemplateDataMeta {
                    id: id.clone(),
                    title: title.clone(),
                    desc: short.clone(),
                    category: category.clone(),
                    author: author.clone(),
                    dates: dates.clone(),
                    thumbnail: thumbnail.clone(),
                    tags: tags.clone(),
                },
                content: match fetch_page_ish_content(postcontent).await.unwrap_html() {
                    RenderrerResponse::Ok(s) => s,
                    _ => return RenderrerResponse::Error,
                },
            },
            _ => todo!("Implement fetching content for postlists."),
        };

        let outerhtml: String = {
            let cwd: PathBuf = std::env::current_dir().unwrap();
            let template_path = cwd.join(
                "cynthiaFiles/templates/".to_owned()
                    + &*localscene.kind.clone()
                    + "/"
                    + &*localscene.template.clone()
                    + ".hbs",
            );
            if !template_path.exists() {
                error!("Template file '{}' not found.", template_path.display());
                return RenderrerResponse::Error;
            }
            // A fallback function that uses the builtin handlebars renderer.
            let builtin_handlebars = || {
                let mut template = Handlebars::new();
                // streq helper
                // This helper checks if two strings are equal.
                // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
                handlebars_helper!(streq: |x: str, y: str| x == y);
                template.register_helper("streq", Box::new(streq));
                match template.register_template_file("base", template_path.clone()) {
                    Ok(g) => g,
                    Err(e) => {
                        error!(
                            "Error reading template file '{}':\n\n{}",
                            template_path.display(),
                            e.to_string().bright_red()
                        );
                        return RenderrerResponse::Error;
                    }
                };
                match template.render("base", &pageish_template_data.meta) {
                    Ok(a) => RenderrerResponse::Ok(a),
                    Err(e) => {
                        error!(
                            "Error rendering template file '{}':\n\n{}",
                            template_path.display(),
                            e.to_string().bright_red()
                        );
                        RenderrerResponse::Error
                    }
                }
            };
            let mut htmlbody: String = if !cfg!(feature = "js_runtime") {
                // Fall back to builtin handlebars if the js_runtime feature is not enabled.
                if let RenderrerResponse::Ok(a) = builtin_handlebars() {
                    a
                } else {
                    return RenderrerResponse::Error;
                }
            } else if let crate::externalpluginservers::EPSResponseBody::OkString { value } =
                crate::externalpluginservers::contact_eps(
                    server_context_mutex.clone(),
                    EPSRequestBody::ContentRenderRequest {
                        template_path: template_path.to_string_lossy().parse().unwrap(),
                        template_data: pageish_template_data.clone(),
                    },
                )
                .await
            {
                value
            } else {
                warn!("External Javascript Runtime failed to render the content. Retrying with basic builtin rendering.");
                // Fall back to builtin handlebars if the external plugin server fails.
                if let RenderrerResponse::Ok(a) = builtin_handlebars() {
                    a
                } else {
                    return RenderrerResponse::Error;
                }
            };
            let version = env!("CARGO_PKG_VERSION");
            let mut head = String::new();
            head.push_str("\n\t<head>");
            head.push_str("\n\t\t<meta charset=\"utf-8\" />");
            head.push_str(
                format!("\n\t\t<title>{}</title>", pageish_template_data.meta.title).as_str(),
            );
            head.push_str("\n\t\t<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />");
            head.push_str("\n\t\t<meta name=\"generator\" content=\"strawmelonjuice-Cynthia\" />");
            head.push_str("\n\t\t<meta name=\"robots\" content=\"index, follow\" />");
            if let Some(stylefile) = localscene.stylesheet {
                let path: PathBuf = std::env::current_dir()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .join("cynthiaFiles/assets/")
                    .join(stylefile);
                if path.exists() {
                    let css = inlines::inline_css(path, server_context_mutex.clone()).await;
                    head.push_str(&css);
                } else {
                    error!("Stylesheet file '{}' not found.", path.display());
                    return RenderrerResponse::Error;
                }
            }
            head.push_str(
&format!("<script>const cynthia = {{version: '{}', publicationdata: JSON.parse(`{}`), kind: '{}'}};</script>",
                version,
    serde_json::to_string(&pageish_template_data.meta.clone()).unwrap(),
                localscene.kind)


            );
            if let Some(script) = localscene.script {
                let path: PathBuf = std::env::current_dir()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .join("cynthiaFiles/assets/")
                    .join(script);
                if path.exists() {
                    let d = inlines::inline_js(path, server_context_mutex.clone()).await;
                    htmlbody.push_str(&d);
                } else {
                    error!("Script file '{}' not found.", path.display());
                    return RenderrerResponse::Error;
                }
            }
            if let Some(author) = pageish_template_data.meta.author {
                if let Some(author_name) = author.name {
                    head.push_str(&format!(
                        "\n\t\t<meta name=\"author\" content=\"{}\" />",
                        author_name
                    ));
                }
            }
            if let Some(category) = pageish_template_data.meta.category {
                head.push_str(&format!(
                    "\n\t\t<meta name=\"category\" content=\"{}\" />",
                    category
                ));
            }
            if let Some(desc) = pageish_template_data.meta.desc {
                head.push_str(&format!(
                    "\n\t\t<meta name=\"description\" content=\"{}\" />",
                    desc
                ));
            }
            if let Some(thumbnail) = pageish_template_data.meta.thumbnail {
                head.push_str(&format!(
                    "\n\t\t<meta property=\"og:image\" content=\"{}\" />",
                    thumbnail
                ));
            }
            head.push_str("\n\t</head>");
            let docurl = "https://github.com/strawmelonjuice/CynthiaWebsiteEngine";
            format!(
                "<!DOCTYPE html>\n<html>\n<!--\n\nGenerated and hosted through Cynthia v{version}, by Strawmelonjuice.\nAlso see:	<{docurl}>\n-->\n{head}\n<body>{htmlbody}</body></html>",
            )
        };

        // content.unwrap().unwrap_html();
        RenderrerResponse::Ok(outerhtml)
    }
    fn fetch_scene(publication: CynthiaPublication, config: CynthiaConfClone) -> Option<Scene> {
        let scene = publication.get_scene_name();
        match scene {
            Some(s) => {
                let fetched_scene = config.scenes.get_by_name(s.as_str());
                if fetched_scene.is_none() {
                    error!("Scene \"{}\" not found in the configuration file.", s);
                    None
                } else {
                    fetched_scene
                }
            }
            None => {
                let fetched_scene = config.scenes.get_default();
                Some(fetched_scene)
            }
        }
    }

    #[derive(Debug)]
    enum FetchedContent {
        Error,
        Ok(ContentType),
    }

    impl FetchedContent {
        fn unwrap_html(self) -> RenderrerResponse {
            match self {
                FetchedContent::Ok(c) => match c {
                    Html(h) => RenderrerResponse::Ok(h),
                    _ => {
                        error!("An error occurred while unwrapping the content.");
                        RenderrerResponse::Error
                    }
                },
                FetchedContent::Error => {
                    error!("An error occurred while unwrapping the content.");
                    RenderrerResponse::Error
                }
            }
        }
    }
    struct ContentSource {
        inner: String,
        target_type: ContentType,
    }
    #[doc = "Fetches the content of a pageish (a post or a page) publication."]
    async fn fetch_page_ish_content(content: PublicationContent) -> FetchedContent {
        let content_output = match content {
            PublicationContent::Inline(c) => ContentSource {
                inner: c.get_inner(),
                target_type: c,
            },
            PublicationContent::External { source } => {
                let a = reqwest::get(source.get_inner()).await;
                let output = match a {
                    Ok(w) => match w.text().await {
                        Ok(o) => o,
                        Err(e) => {
                            error!(
                                "Could not fetch external content from {}\n\n{e}",
                                source.get_inner()
                            );
                            return FetchedContent::Error;
                        }
                    },
                    Err(e) => {
                        error!(
                            "Could not fetch external content from {}\n\n{e}",
                            source.get_inner()
                        );
                        return FetchedContent::Error;
                    }
                };
                ContentSource {
                    inner: output,
                    target_type: source,
                }
            }
            PublicationContent::Local { source } => {
                let output = {
                    let mut v = String::from("./cynthiaFiles/publications/");
                    v.push_str(&source.get_inner());
                    if Path::new(v.as_str()).exists() {
                        match fs::read_to_string(v.clone()) {
                            Ok(t) => t,
                            Err(e) => {
                                error!("Could not read local content at {}\n\n{e}", v);
                                return FetchedContent::Error;
                            }
                        }
                    } else {
                        error!("Could not find local content at {}", v);
                        return FetchedContent::Error;
                    }
                };
                ContentSource {
                    inner: output,
                    target_type: source,
                }
            }
        };
        let contenttype = match content_output.target_type {
            Html(_) => Html(content_output.inner),
            ContentType::Markdown(_) => {
                let html = match markdown::to_html_with_options(
                    content_output.inner.as_str(),
                    &markdown::Options::gfm(),
                ) {
                    Ok(html) => html,
                    Err(_) => {
                        error!("An error occurred while rendering the markdown.");
                        return FetchedContent::Error;
                    }
                };
                Html(html)
            }
            ContentType::PlainText(_) => {
                Html("<pre>".to_owned() + content_output.inner.as_str() + "</pre>")
            }
        };

        FetchedContent::Ok(contenttype)
    }
}
#[cfg(feature = "js_runtime")]
mod inlines {
    use crate::{LockCallback, ServerContext};
    use actix_web::web::Data;
    use colored::Colorize;
    use log::{debug, error, warn};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub(crate) async fn inline_js(
        scriptfile: PathBuf,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let config_clone = server_context_mutex
            .lock_callback(|a| {
                a.request_count += 1;
                a.config.clone()
            })
            .await;
        let embed_id = scriptfile.clone().to_str().unwrap().to_string();
        let jscachelifetime: u64 = config_clone.cache.lifetimes.javascript;
        let cache_result = server_context_mutex
            .lock_callback(|servercontext| servercontext.get_cache(&embed_id, jscachelifetime))
            .await;
        match cache_result {
            Some(o) => {
                let d = std::str::from_utf8(&o.0).unwrap().to_string();
                return format!(
                    "<script>\n\r// Minified internally by Cynthia using Terser\n\n{d}\n\n\r// Cached after minifying, so might be somewhat behind.\n\r</script>");
            }
            None => {
                let xargs: Vec<&str>;
                let scri = scriptfile.clone();
                let scr = scri.to_str().unwrap();
                let runner = {
                    if config_clone.runtimes.ext_js_rt.as_str().contains("bun") {
                        xargs = [
                            "terser",
                            scr,
                            "--compress",
                            "--keep-fnames",
                            "--keep-classnames",
                        ]
                        .to_vec();

                        "bunx"
                    } else {
                        xargs = [
                            "--yes",
                            "terser",
                            scr,
                            "--compress",
                            "--keep-fnames",
                            "--keep-classnames",
                        ]
                        .to_vec();

                        "npx"
                    }
                };

                debug!("Running Terser in {}", runner.purple());
                match std::process::Command::new(runner)
                    .args(xargs.clone())
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            let d = format!("{}", String::from_utf8_lossy(&output.stdout));
                            debug!("Minified JS: {}", d);
                            {
                                let mut server_context = server_context_mutex.lock().await;
                                server_context
                                    .store_cache_async(&embed_id, d.as_bytes(), jscachelifetime)
                                    .await
                                    .unwrap();
                            };
                            return format!(
                                "<script>\n\r// Minified internally by Cynthia using Terser\n\n{d}\n\n\r// Cached after minifying, so might be somewhat behind.\n\r</script>");
                        } else {
                            warn!(
                                "Failed running Terser in {}, couldn't minify to embed JS.",
                                config_clone.runtimes.ext_js_rt.as_str().purple()
                            );
                            println!("Ran command \"{} {}\"", runner.purple(), {
                                let mut s = String::new();
                                for a in &xargs {
                                    s.push_str(a);
                                    s.push(' ');
                                }
                                s
                            })
                        }
                    }
                    Err(why) => {
                        error!(
                            "Failed running CleanCSS in {}, couldn't minify to embed JS: {}",
                            config_clone.runtimes.ext_js_rt.as_str().purple(),
                            why
                        );
                    }
                }
            }
        };
        warn!("Scriptfile could not be minified, so was instead inlined 1:1.");
        //     If we got here, we couldn't minify the JS.
        let file_content = fs::read_to_string(scriptfile).unwrap_or_default();
        format!("<script>\n// Scriptfile could not be minified, so was instead inlined 1:1. \n\n{}</script>", file_content)
    }

    pub(crate) async fn inline_css(
        stylefile: PathBuf,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let config_clone = server_context_mutex
            .lock_callback(|a| {
                a.request_count += 1;
                a.config.clone()
            })
            .await;
        let embed_id = stylefile.clone().to_str().unwrap().to_string();
        let csscachelifetime: u64 = config_clone.cache.lifetimes.stylesheets;
        let cache_result = server_context_mutex
            .lock_callback(|servercontext| servercontext.get_cache(&embed_id, csscachelifetime))
            .await;
        match cache_result {
            Some(o) => {
                let d = std::str::from_utf8(&o.0).unwrap().to_string();
                return format!(
                    "\n\t\t<style>\n\n\t\t\t/* Minified internally by Cynthia using clean-css */\n\n\t\t\t{d}\n\n\t\t\t/* Cached after minifying, so might be somewhat behind. */\n\t\t</style>");
            }
            None => {
                let xargs: Vec<&str>;
                let styf = stylefile.clone();
                let stf = styf.to_str().unwrap();
                let runner = {
                    if config_clone.runtimes.ext_js_rt.as_str().contains("bun") {
                        xargs = ["clean-css-cli@4", "-O2", "--inline", "none", stf].to_vec();

                        "bunx"
                    } else {
                        xargs =
                            ["--yes", "clean-css-cli@4", "-O2", "--inline", "none", stf].to_vec();

                        "npx"
                    }
                };
                debug!("Running CleanCSS in {}", runner.purple());
                match std::process::Command::new(runner)
                    .args(xargs.clone())
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            let d = format!("{}", String::from_utf8_lossy(&output.stdout));
                            debug!("Minified JS: {}", d);
                            {
                                let mut server_context = server_context_mutex.lock().await;
                                server_context
                                    .store_cache_async(&embed_id, d.as_bytes(), csscachelifetime)
                                    .await
                                    .unwrap();
                            }
                            return format!(
                                    "\n\t\t<style>\n\n\t\t\t/* Minified internally by Cynthia using clean-css */\n\n\t\t\t{d}\n\n\t\t\t/* Cached after minifying, so might be somewhat behind. */\n\t\t</style>");
                        }
                    }
                    Err(why) => {
                        error!(
                            "Failed running CleanCSS in {}, couldn't minify to embed CSS: {}",
                            config_clone.runtimes.ext_js_rt.as_str().purple(),
                            why
                        );
                        debug!("Ran command \"{} {}\"", runner.purple(), {
                            let mut s = String::new();
                            for a in &xargs {
                                s.push_str(a);
                                s.push(' ');
                            }
                            s
                        });
                    }
                }
            }
        };
        warn!("Stylefile could not be minified, so was instead inlined 1:1.");
        //     If we got here, we couldn't minify the CSS.
        let file_content = fs::read_to_string(stylefile).unwrap_or_default();
        format!("<style>\n/* Stylefile could not be minified, so was instead inlined 1:1. */\n\n{}</style>", file_content)
    }
}

#[cfg(not(feature = "js_runtime"))]
mod inlines {
    pub(crate) async fn inline_js(
        scriptfile: PathBuf,
        _server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let file_content = fs::read_to_string(scriptfile).unwrap_or(String::new());
        format!("<script>{}</script>", file_content)
    }
    pub(crate) async fn inline_css(
        stylefile: PathBuf,
        _server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let file_content = fs::read_to_string(stylefile).unwrap_or(String::new());
        format!("<style>{}</style>", file_content)
    }
}
