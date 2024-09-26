#![deny(elided_lifetimes_in_paths)]
#[macro_use] extern crate rocket;

use libpixiv::PixivAppClient;
use maud::{html, DOCTYPE};
use rocket::{http::Status, response::content::RawHtml, tokio::{self, sync::Mutex}, State};
use std::{sync::Arc, thread::sleep, time::Duration};

struct Metadata {
    pub image: String,
    pub title: String,
    pub desc: String,
}

#[get("/<path..>")]
async fn handle_route(state: &State<Option<Arc<Mutex<PixivAppClient>>>>, path: std::path::PathBuf) -> Result<RawHtml<String>, Status> {
    let target = format!("https://pixiv.net/{}", path.display());

    if let Some(id) = path.file_name() {
        let meta = fetch_illust(state, id.to_str().unwrap().parse::<u32>().unwrap());
        Ok(RawHtml(create_page(&target, &meta.await.unwrap()).await.unwrap()))
    } else {
        Err(Status::InternalServerError)
    }

}

async fn fetch_illust(client: &Option<Arc<Mutex<PixivAppClient>>>, illust_id: u32) -> Option<Metadata> {
    if let Some(client) = client {
        if let Ok(illust) = client.lock().await.illust_details(illust_id).await {
            let image = if illust.page_count == 1 { illust.meta_single_page.unwrap().original_image_url.unwrap() } else { illust.meta_pages[0].image_urls.large.clone() };
            return Some(Metadata {
                image: image.as_str().replace("pximg.net", "fixiv.net"),
                title: illust.title,
                desc: illust.caption,
            })
        }
    }

    None
}

async fn create_page(source: &str, meta: &Metadata) -> Result<String, Status> {
    Ok(html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";

                meta property="og:title" content=(meta.title);
                meta property="og:image" content=(meta.image);
                meta property="og:url" content=(source);
                meta property="og:type" content="article";
                meta property="og:site_name" content="pixiv";
                meta property="og:description" content=(meta.desc);

                meta property="twitter:card" content="summary_large_image";
                meta property="twitter:site" content="@pixiv";
                meta property="twitter:url" content=(source);
                meta property="twitter:title" content=(meta.title);
                meta property="twitter:description" content=(meta.desc);
                meta property="twitter:image" content=(meta.image);


                meta http-equiv="Refresh" content=(format!("0; url='{}'", source));
            }
            body {
                h1 { "fxpixiv.net" }
                h2 { "your (not yet so) friendly pixiv embed helper" }
                p { "This page will take you to the original one one pixiv - if not, "}
                a href=(source) { "refresh manually" }
            }
        }
    }
    .into_string())
}

#[launch]
async fn launch() -> _ {

    let mut pixiv_client: Option<Arc<Mutex<PixivAppClient>>> = None;

    if let Ok(token) = std::env::var("PIXIV_REFRESH_TOKEN") {
        let mut client = PixivAppClient::new(token);
        client.refresh_token().await; // TODO: refresh token on a regular basis
        pixiv_client = Some(Arc::new(Mutex::new(client)));
    }

    if let Some(pixiv_client) = &pixiv_client {
        let background_client = Arc::clone(&pixiv_client);
        tokio::spawn(async move {
            loop {
                background_client.lock().await.refresh_token().await;
                sleep(Duration::from_secs(40 * 60));
            }
        });
    }

    rocket::build()
        .manage(pixiv_client)
        .mount("/", routes![handle_route])
}
