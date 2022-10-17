use std::sync::Arc;

use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, Pool, Postgres};

use money::Money;

mod db;
mod money;

#[tokio::main]
async fn main() {
    let beer = Item::new(0, "Beer".to_string(), 4000, 100, 90, 1000);
    println!("profit beer: {}", beer.get_profit());
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
        .layer(Extension(Api::new(pool)));

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
        let items = sqlx::query_as!(Item, r#"SELECT id, name, buy_price as "buy_price: Money", sell_price as "sell_price: Money", units_per_buy, amount_in_stock FROM items"#)
            .fetch_all(&*self.pool)
            .await?;
        Ok(items)
    }

    pub async fn insert_new_item(&self, mut item: Item) -> Result<Item, sqlx::Error> {
        let z = sqlx::query!(
            r#"INSERT INTO items (name, buy_price, sell_price, units_per_buy, amount_in_stock) VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
            item.name,
            *item.buy_price,
            *item.sell_price,
            item.units_per_buy,
            item.amount_in_stock,
        )
        .fetch_one(&*self.pool)
        .await?;
        item.id = z.id;
        println!("item: {:?}", item);
        Ok(item)
    }
}

#[derive(Template, Debug)]
#[template(path = "list.html")]
struct ListPage {
    items: Vec<Item>,
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
    fn new(
        id: i32,
        name: String,
        buy_price: impl Into<Money>,
        sell_price: impl Into<Money>,
        units_per_buy: i32,
        amount_in_stock: i32,
    ) -> Item {
        Item {
            id,
            name,
            buy_price: buy_price.into(),
            sell_price: sell_price.into(),
            units_per_buy,
            amount_in_stock,
        }
    }

    fn get_profit(&self) -> Money {
        self.units_per_buy * self.sell_price - self.buy_price
    }
}
