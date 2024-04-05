mod md;
mod web;

#[tokio::main]
async fn main() {
    let post_path = "posts";
    let static_path = "static";
    web::web_serve(post_path, static_path).await;
}
