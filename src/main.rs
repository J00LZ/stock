use std::sync::Arc;

use askama::Template;
use axum::{
    extract::Path,
    http::Method,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, Pool, Postgres};

use money::Money;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod db;
mod money;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();
    db::db_setup(&pool).await;

    let route = Router::new()
        .route("/foo", get(get_all_items))
        .route("/bar", post(insert_item))
        .route("/list", get(list_page))
        .route("/add_item", get(add_item_page).post(add_item_post))
        .route("/increase/:item_id", post(increase))
        .route("/decrease/:item_id", post(decrease))
        .layer(Extension(Api::new(pool)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(vec![Method::GET, Method::POST]),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = ([127, 0, 0, 1], 3000).into();
    axum::Server::bind(&addr)
        .serve(route.into_make_service())
        .await
        .unwrap();
}

async fn get_all_items(Extension(api): Extension<Api>) -> Json<Vec<Item>> {
    let items = api.get_items().await;
    Json(items.unwrap())
}

async fn insert_item(Extension(api): Extension<Api>, item: Json<Item>) -> Json<Item> {
    let item = api.insert_new_item(item.0).await;
    Json(item.unwrap())
}

async fn list_page(Extension(api): Extension<Api>) -> impl IntoResponse {
    let items = api.get_items().await;
    let items = items.unwrap();
    let page = ListPage { items };
    Html(page.render().unwrap())
}

async fn add_item_page() -> impl IntoResponse {
    let page = AddItemPage { item: None };
    Html(page.render().unwrap())
}

#[derive(Deserialize)]
struct ItemForm {
    name: String,
    buy_price: f64,
    sell_price: f64,
    units_per_buy: i32,
}

impl From<ItemForm> for Item {
    fn from(form: ItemForm) -> Self {
        Self {
            id: 0,
            name: form.name,
            buy_price: Money::from(form.buy_price),
            sell_price: Money::from(form.sell_price),
            units_per_buy: form.units_per_buy,
            amount_in_stock: 0,
        }
    }
}

async fn add_item_post(Extension(api): Extension<Api>, item: Form<ItemForm>) -> impl IntoResponse {
    let item = api.insert_new_item(item.0.into()).await;
    let item = item.unwrap();
    let page = AddItemPage { item: Some(item) };
    Html(page.render().unwrap())
}

async fn decrease(Extension(api): Extension<Api>, Path(item_id): Path<i32>) -> Json<Item> {
    let item = api.modify_item(item_id, |i| i.amount_in_stock -= 1).await;
    Json(item)
}

async fn increase(Extension(api): Extension<Api>, Path(item_id): Path<i32>) -> Json<Item> {
    let item = api.modify_item(item_id, |i| i.amount_in_stock += 1).await;
    Json(item)
}

#[derive(Clone)]
struct Api {
    pool: Arc<Pool<Postgres>>,
}

impl Api {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

impl Api {
    pub async fn get_items(&self) -> Result<Vec<Item>, sqlx::Error> {
        let items = sqlx::query_as::<_, Item>(
            r#"SELECT id, name, buy_price, sell_price, units_per_buy, amount_in_stock FROM items"#,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(items)
    }

    pub async fn insert_new_item(&self, item: Item) -> Result<Item, sqlx::Error> {
        let z = sqlx::query_as::<_, Item>(
            r#"INSERT INTO items (name, buy_price, sell_price, units_per_buy, amount_in_stock) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(item.name)
            .bind(item.buy_price)
            .bind(item.sell_price)
            .bind(item.units_per_buy)
            .bind(item.amount_in_stock)
        .fetch_one(&*self.pool)
        .await?;
        println!("item: {:?}", z);
        Ok(z)
    }

    pub async fn modify_item<F>(&self, id: i32, f: F) -> Item
    where
        F: Fn(&mut Item),
    {
        let mut item = self.get_item(id).await.unwrap();
        f(&mut item);
        self.update_item(item).await.unwrap()
    }

    async fn get_item(&self, id: i32) -> Result<Item, sqlx::Error> {
        let item = sqlx::query_as::<_, Item>(
            r#"SELECT id, name, buy_price, sell_price, units_per_buy, amount_in_stock FROM items WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await?;
        Ok(item)
    }

    async fn update_item(&self, item: Item) -> Result<Item, sqlx::Error> {
        let item = sqlx::query_as::<_, Item>(
            r#"UPDATE items SET name = $1, buy_price = $2, sell_price = $3, units_per_buy = $4, amount_in_stock = $5 WHERE id = $6 RETURNING *"#,
        )
        .bind(item.name)
            .bind(item.buy_price)
            .bind(item.sell_price)
            .bind(item.units_per_buy)
            .bind(item.amount_in_stock)
        .bind(item.id)
        .fetch_one(&*self.pool)
        .await?;
        Ok(item)
    }
}

#[derive(Template, Debug)]
#[template(path = "list.html")]
struct ListPage {
    items: Vec<Item>,
}

#[derive(Template, Debug)]
#[template(path = "add_item.html")]
struct AddItemPage {
    item: Option<Item>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct StockChange {
    item_id: u32,
    amount: i32,
    timestamp: i64,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct Item {
    #[serde(default)]
    id: i32,
    name: String,
    buy_price: Money,
    sell_price: Money,
    units_per_buy: i32,
    amount_in_stock: i32,
}

impl Item {
    fn get_profit(&self) -> Money {
        self.units_per_buy * self.sell_price - self.buy_price
    }
}
