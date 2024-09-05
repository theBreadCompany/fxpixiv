#[macro_use] extern crate rocket;
#[macro_use] extern crate json;
#[macro_use] extern crate markup5ever;


use rocket::response::content::RawHtml;
use rocket::http::Status;
use rocket::http::uri::Segments;
use reqwest::Client;
use scraper::{Html, Selector};
use html5ever::tree_builder::TreeSink;
use tendril::Tendril;
use markup5ever::{Attribute, QualName, LocalName};
use markup5ever::interface::tree_builder::TreeSink;

#[get("/<path..>")]
async fn handle_route(path: std::path::PathBuf) -> Result<RawHtml<String>, Status> {
    let target = format!("https://pixiv.net/{}", path.display());

    let html = match fetch_content(&target).await {
        Ok(html) => html,
        Err(err) => return Err(err), 
    };

    let modified = change_meta(&html).await.unwrap();

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

async fn change_meta(html: &String) -> Result<String, Status> {
    let dom = Html::parse_document(html);
    let data_selector = match Selector::parse(r#"meta[name="preload-data"]"#) {
        Ok(sel) => sel,
        Err(_) => return Ok(html.to_string()),
    };
    let data_meta = match dom.select(&data_selector).next() {
        Some(meta) => meta,
        None => return Ok(html.to_string()),
    };

    let illust = json::parse(data_meta.value().attr("content").unwrap());
    let target_url = illust.unwrap()["illust"].entries().next().unwrap().1["urls"]["regular"].as_str().unwrap();


    let image_selector = match Selector::parse(r#"meta[name="og:image"]"#) {
        Ok(sel) => sel,
        Err(_) => return Ok(html.to_string()),
    };
    let image_meta = match dom.select(&image_selector).next() {
        Some(meta) => meta,
        None => return Ok(html.to_string()),
    };

    let parent = image_meta.parent().unwrap();
    dom.remove_from_parent(image_meta);
    dom.create_element("meta", vec![
        Attribute { name: QualName::new(
            None,
            ns!(html),
            LocalName::from("name"),
        ),
            value: Tendril::from("og:image")
        }, 
        Attribute { name: QualName::new(
            None,
            ns!(html),
            LocalName::from("content"),
        ),
            value: Tendril::from(target_url)
        } 
    ]);


    Ok(html.to_string())
}

#[launch]
fn launch() -> _ {
    rocket::build().mount("/", routes![handle_route])
}
