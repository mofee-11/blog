use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use askama_warp::Template;
use warp;
use warp::reply::Reply;
use warp::Filter;

use crate::md;
use crate::md::Page;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    links: String,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    markdown: String,
}

static POST_PATH: OnceLock<PathBuf> = OnceLock::new();
static STATIC_PATH: OnceLock<PathBuf> = OnceLock::new();

pub async fn web_serve<P: AsRef<Path>>(post_path: P, static_path: P) {
    POST_PATH.set(PathBuf::from(post_path.as_ref())).unwrap();
    STATIC_PATH
        .set(PathBuf::from(static_path.as_ref()))
        .unwrap();

    let home = warp::path::end().then(home_handler);
    let post = warp::path!("post" / String).then(post_handler);
    let static_file = warp::fs::dir(STATIC_PATH.get().unwrap());

    let route = home.or(post).or(static_file);

    for port in 3000..65535 {
        if is_port_free(port) {
            println!("serving on the http://127.0.0.1:{}", port);
            warp::serve(route).run(([127, 0, 0, 1], port)).await;
            break;
        }
    }

    println!("no tcp port available");
}

fn is_port_free(port: u16) -> bool {
    if let Ok(_) = std::net::TcpListener::bind(format!("127.0.0.1:{}", port)) {
        true
    } else {
        false
    }
}

async fn home_handler() -> impl Reply {
    let posts = md::Posts::new(POST_PATH.get().unwrap()).unwrap();

    let mut menu = String::new();
    let mut year = String::new();
    let mut month = String::new();

    posts.into_iter().for_each(|page| {
        let id = page.id();

        if page.date.format("%Y").to_string() != year {
            year = page.date.format("%Y").to_string();
            menu.push_str(&format!("<div class=\"year\">{}</div>", &year))
        }

        if page.date.format("%m").to_string() != month {
            month = page.date.format("%m").to_string();
            menu.push_str(&format!("<div class=\"month\">{}</div>", &month))
        }

        let html = format!(
            "<div class=\"item\"><a href=\"/post/{}\">{}</a></div>",
            id,
            page.title()
        );
        menu.push_str(&html)
    });

    HomeTemplate { links: menu }
}

async fn post_handler(id: String) -> impl Reply {
    let mut post_path = PathBuf::from(POST_PATH.get().unwrap());
    post_path.push(id);
    post_path.set_extension("md");

    let page = match Page::new(post_path) {
        Some(p) => p,
        None => {
            return PostTemplate {
                markdown: "<h1>404</h1>".to_owned(),
            }
        }
    };

    PostTemplate {
        markdown: page.md_html(),
    }
}
