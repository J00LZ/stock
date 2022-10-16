use std::time::Instant;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let beer = Item::new(0, "Beer".to_string(), 4000, 100, 90, 1000);
    println!("profit beer: {}", beer.get_profit());
    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect("postgresql://postgres:postgres@localhost:5432/postgres")
    //     .await
    //     .unwrap();

    
}

struct StockChange {
    item_id: u32,
    amount: i32,
    timestamp: Instant,
}

struct Item {
    id: u32,
    name: String,
    buy_price: u32,
    sell_price: u32,
    units_per_buy: u32,
    amount_in_stock: u32,
}

impl Item {
    fn new(
        id: u32,
        name: String,
        buy_price: u32,
        sell_price: u32,
        units_per_buy: u32,
        amount_in_stock: u32,
    ) -> Item {
        Item {
            id,
            name,
            buy_price,
            sell_price,
            units_per_buy,
            amount_in_stock,
        }
    }

    fn get_profit(&self) -> u32 {
        self.units_per_buy * self.sell_price - self.buy_price
    }
}
