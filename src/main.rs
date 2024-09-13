#[macro_use] extern crate rocket;

use rocket::response::content::RawHtml;
use rocket::http::Status;
use reqwest::Client;
use scraper::{Html, Selector};
use maud::{html, DOCTYPE};

#[get("/<path..>")]
async fn handle_route(path: std::path::PathBuf) -> Result<RawHtml<String>, Status> {
    let target = format!("https://pixiv.net/{}", path.display());

    let html = match fetch_content(&target).await {
        Ok(html) => html,
        Err(err) => return Err(err), 
    };

    let modified = create_page(&target, &html).await.unwrap();

    Ok(RawHtml(modified))
}

async fn fetch_content(url: &String) -> Result<String, Status> {
    println!("{}", url);
    let client = Client::new();
    let response = match client.get(url).send().await {
        Ok(resp) => resp,
        Err(_) => return Err(Status::BadGateway),
    };

    let html = match response.text().await {
        Ok(text) => text,
        Err(_) => return Err(Status::InternalServerError),
    };
    
    Ok(html)
}

async fn create_page(source: &String, html: &String) -> Result<String, Status> {
    let dom = Html::parse_document(html);

    let data_selector = match Selector::parse(r#"meta[name="preload-data"]"#) {
        Ok(sel) => sel,
        Err(_) => return Err(Status::InternalServerError),
    };
    let data_meta = match dom.select(&data_selector).next() {
        Some(meta) => meta,
        None => return Err(Status::InternalServerError),
    };

    let illust = json::parse(data_meta.value().attr("content").unwrap()).unwrap();
    let image = match illust["illust"].entries().next() {
        Some (j) => j.1["urls"]["regular"].as_str().unwrap().replace("pximg.net", "thebread.dev"),
        None => "https://http.cat/images/501.jpg".to_string(),
    };

    let title_selector = match Selector::parse(r#"meta[property="og:title"]"#) {
        Ok(sel) => sel,
        Err(_) => return Err(Status::InternalServerError),
    };
    let title_meta = match dom.select(&title_selector).next() {
        Some(meta) => meta.attr("content").unwrap(),
        None => "unknown title"
    };
    let desc_selector = match Selector::parse(r#"meta[property="og:description"]"#) {
        Ok(sel) => sel,
        Err(_) => return Err(Status::InternalServerError),
    };
    let desc_meta = match dom.select(&desc_selector).next() {
        Some(meta) => meta.attr("content").unwrap(),
        None => "unknown title"
    };

    Ok(html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";

                meta property="og:title" content=(title_meta);
                meta property="og:image" content=(image);
                meta property="og:url" content=(source);
                meta property="og:type" content="article";
                meta property="og:site_name" content="pixiv";
                meta property="og:description" content=(desc_meta);

                meta property="twitter:card" content="summary_large_image";
                meta property="twitter:site" content="@pixiv";
                meta property="twitter:url" content=(source);
                meta property="twitter:title" content=(title_meta);
                meta property="twitter:description" content=(desc_meta);
                meta property="twitter:image" content=(image);


                meta http-equiv="Refresh" content=(format!("0; url='{}'", source));
            }
            body {
                h1 { "fxpixiv.net" }
                h2 { "your (not yet so) friendly pixiv embed helper" }
                p { "This page will take you to the original one one pixiv - if not, "}
                a href=(source) { "refresh manually" }
            }
        }
    }.into_string())
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/", routes![handle_route])
}
