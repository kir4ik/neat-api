// https://blog.logrocket.com/creating-a-rest-api-in-rust-with-warp/

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use warp::{http, Filter};

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let add_items = warp::post()
        .and(warp::path::end())
        .and(json_body())
        .and(store_filter.clone())
        .and_then(add_grocery_list_item);

    let get_items = warp::get()
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_grocery_list);

    let del_item = warp::delete()
        .and(warp::path!(String))
        .and(store_filter.clone())
        .and_then(del_grocery_list);

    let groceries_routes = warp::path("v1")
        .and(warp::path("groceries"))
        .and(add_items.or(get_items).or(del_item));

    warp::serve(groceries_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn json_body() -> impl Filter<Extract = (Item,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn add_grocery_list_item(
    item: Item,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.grocery_list.write().insert(item.name, item.quantity);

    Ok(warp::reply::with_status(
        "Added item to the grocery list",
        http::StatusCode::CREATED,
    ))
}

async fn get_grocery_list(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut result = HashMap::new();
    let r = store.grocery_list.read();

    for (key, value) in r.iter() {
        result.insert(key, value);
    }

    Ok(warp::reply::json(&result))
}

async fn del_grocery_list(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    store.grocery_list.write().remove(&id);

    Ok(warp::reply::with_status(
        format!("Removed item {}", id),
        http::StatusCode::OK,
    ))
}

type Items = HashMap<String, i32>;

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    name: String,
    quantity: i32,
}

#[derive(Clone)]
struct Store {
    grocery_list: Arc<RwLock<Items>>,
}

impl Store {
    fn new() -> Self {
        Store {
            grocery_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
