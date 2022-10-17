use sqlx::{Executor, FromRow, Pool, Postgres};

pub async fn db_setup(pool: &Pool<Postgres>) {
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS versions (
            number INT PRIMARY KEY
        )
        "#,
    )
    .await
    .unwrap();

    let version = sqlx::query_as::<_, Version>("SELECT number FROM versions")
        .fetch_optional(pool)
        .await
        .unwrap();
    let version = if let Some(v) = version { v.number } else { 0 };
    match version {
        0 => {
            pool.execute(
                r#"
                CREATE TABLE IF NOT EXISTS items (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    buy_price INT NOT NULL,
                    sell_price INT NOT NULL,
                    units_per_buy INT NOT NULL,
                    amount_in_stock INT NOT NULL
                )
                "#,
            )
            .await
            .unwrap();
            pool.execute(
                r#"
                CREATE TABLE IF NOT EXISTS stock_changes (
                    id SERIAL PRIMARY KEY,
                    item_id INT NOT NULL,
                    amount INT NOT NULL,
                    timestamp TIMESTAMP NOT NULL
                )
                "#,
            )
            .await
            .unwrap();
            pool.execute(
                r#"
                INSERT INTO versions (number) VALUES (1)
                "#,
            )
            .await
            .unwrap();
        }
        _ => {}
    }
}

#[derive(FromRow)]
struct Version {
    number: i32,
}
