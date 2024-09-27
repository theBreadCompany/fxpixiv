#![deny(elided_lifetimes_in_paths)]
#[macro_use]
extern crate rocket;

use std::{str::FromStr, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use libpixiv::PixivAppClient;
use maud::{html, DOCTYPE};
use rocket::{fairing::AdHoc, http::Status, response::content::RawHtml, tokio, tokio::sync::Mutex, State};
use rocket_db_pools::{
    Database,
    {sqlx, sqlx::Row},
};

#[derive(Database)]
#[database("pixiv")]
struct PixivDB(sqlx::SqlitePool);

struct Metadata {
    pub image: String,
    pub title: String,
    pub desc: String,
}

#[get("/<path..>")]
async fn handle_route(
    client: &State<Option<Arc<Mutex<PixivAppClient>>>>,
    db: &State<PixivDB>,
    path: std::path::PathBuf,
) -> Result<RawHtml<String>, Status> {
    let target = format!("https://pixiv.net/{}", path.display());

    if let Some(id) = path.file_name() {
        let meta = fetch_illust(client, &db, id.to_str().unwrap().parse::<u32>().unwrap());
        Ok(RawHtml(
            create_page(&target, &meta.await.unwrap()).await.unwrap(),
        ))
    } else {
        Err(Status::InternalServerError)
    }
}

async fn fetch_illust(
    client: &Option<Arc<Mutex<PixivAppClient>>>,
    db: &PixivDB,
    illust_id: u32,
) -> Option<Metadata> {
    if let Ok(row) = sqlx::query(
        "
        SELECT p.large, i.title, i.desc, i.expires_on
        FROM Illustrations i
        JOIN IllustrationPages p ON p.illust_id = i.id
        WHERE p.page_number = 0 AND i.id = ?
        ",
    )
    .bind(illust_id)
    .fetch_one(&db.0)
    .await
    {
        if Utc::now() >= DateTime::<Utc>::from_str(row.get(0)).unwrap() {
            return Some(Metadata {
                image: row.get(0),
                title: row.get(1),
                desc: row.get(2),
            });
        } else {
            let _ = sqlx::query(
                "
            DELETE FROM Illustrations
            WHERE id = $1
            ",
            )
            .bind(illust_id)
            .execute(&db.0)
            .await;
        }
    };

    if let Some(client) = client {
        if let Ok(illust) = client.lock().await.illust_details(illust_id).await {
            let user_query = "
            INSERT OR REPLACE INTO User (id, name) 
            VALUES ($1, $2) 
            RETURNING id
            ";
            let _ = sqlx::query(user_query)
                .bind(illust.user.id)
                .bind(illust.user.name)
                .fetch_one(&db.0)
                .await
                .ok()
                .expect("failed to insert user");

            let illust_query = "
            INSERT OR REPLACE INTO Illustrations (id, title, description, user, expires_on)
            VALUES ($1, $2, $3, $4, $5)
            ";
            let _ = sqlx::query(&illust_query)
                .bind(illust.id)
                .bind(&illust.title)
                .bind(&illust.caption)
                .bind(illust.user.id)
                .bind((Utc::now() + Duration::days(7)).to_string())
                .execute(&db.0)
                .await
                .expect("failed to insert illustration");

            let pages_query = "
            INSERT OR REPLACE INTO IllustrationPages (square_medium, medium, large, original, page_number, illust_id) 
            VALUES ($1, $2, $3, $4, $5, $6)
            ";
            if illust.meta_pages.is_empty() {
                let _ = sqlx::query(pages_query)
                    .bind(illust.image_urls.square_medium)
                    .bind(illust.image_urls.medium)
                    .bind(illust.image_urls.large)
                    .bind(
                        &illust
                            .meta_single_page
                            .as_ref()
                            .unwrap()
                            .original_image_url
                            .clone()
                            .unwrap(),
                    )
                    .bind(0)
                    .bind(illust.id)
                    .execute(&db.0)
                    .await
                    .expect("failed to insert page");
            } else {
                let _ = illust.meta_pages.iter().enumerate().map(|page| async move {
                    let _ = sqlx::query(pages_query)
                        .bind(&page.1.image_urls.square_medium)
                        .bind(&page.1.image_urls.medium)
                        .bind(&page.1.image_urls.large)
                        .bind(&page.1.image_urls.original)
                        .bind(page.0.to_string())
                        .bind(illust.id)
                        .execute(&db.0)
                        .await
                        .expect("failed to insert page");
                });
            }

            let image = if illust.page_count == 1 {
                illust.meta_single_page.unwrap().original_image_url.unwrap()
            } else {
                illust.meta_pages[0].image_urls.large.clone()
            };

            return Some(Metadata {
                image: image.as_str().replace("pximg.net", "fixiv.net"),
                title: illust.title,
                desc: illust.caption,
            });
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

    let mut pixiv_client: Option<std::sync::Arc<tokio::sync::Mutex<PixivAppClient>>> = None;

    if let Ok(token) = std::env::var("PIXIV_REFRESH_TOKEN") {
        let mut client = PixivAppClient::new(token);
        client.refresh_token().await; // TODO: refresh token on a regular basis
        pixiv_client = Some(std::sync::Arc::new(tokio::sync::Mutex::new(client)));
    }

    if let Some(pixiv_client) = &pixiv_client {
        let background_client = std::sync::Arc::clone(&pixiv_client);
        tokio::spawn(async move {
            loop {
                background_client.lock().await.refresh_token().await;
                std::thread::sleep(std::time::Duration::from_secs(40 * 60));
            }
        });
    }

    rocket::build()
        .manage(pixiv_client)
        .attach(PixivDB::init())
        .attach(AdHoc::on_ignite("Database Init", |rocket| {
            Box::pin(async {
                let db = PixivDB::fetch(&rocket).expect("Database connection failed");
                sqlx::query(
                    "
                    CREATE TABLE IF NOT EXISTS User (
                        id INTEGER UNIQUE PRIMARY KEY,
                        name TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS Illustrations (
                        id INTEGER UNIQUE PRIMARY KEY,
                        title TEXT NOT NULL,
                        description TEXT NOT NULL,
                        user INTEGER NOT NULL,
                        expires_on TIMESTAMP,
                        FOREIGN KEY (user) REFERENCES User(id)
                    );
                    CREATE TABLE IF NOT EXISTS IllustrationPages (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        illust_id INTEGER NOT NULL,
                        page_number INTEGER NOT NULL,
                        square_medium TEXT NOT NULL,
                        medium TEXT NOT NULL,
                        large TEXT NOT NULL,
                        original TEXT NOT NULL,
                        FOREIGN KEY (illust_id) REFERENCES Illustrations(id)
                    );
                    ",
                )
                .execute(&db.0)
                .await
                .expect("Database init failed!");
                rocket
            })
        }))
        .mount("/", routes![handle_route])
}
