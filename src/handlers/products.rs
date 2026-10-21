use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    AppState,
    error::{Error, Result},
    models::{NewProduct, Product},
    schema::products,
};

async fn get_product(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Product>> {
    let pool = state.pool;

    let mut conn = pool.get().await?;

    // TODO : properly parse and return if not an int
    let id: i32 = slug.parse().unwrap();

    let product: Product = Product::query()
        .filter(products::id.eq(id))
        .first(&mut conn)
        .await
        .unwrap();

    Ok(Json(product))
}

// TODO : make sure only admins can access this API
async fn create_product(
    State(state): State<AppState>,
    Json(new_product): Json<NewProduct>,
) -> Result<Json<Product>> {
    let pool = state.pool;

    let mut conn = pool.get().await?;

    let res: Vec<Product> = Product::query().load(&mut conn).await?;

    if res.iter().find(|&x| x.name == new_product.name).is_some() {
        return Err(Error::Model(crate::models::ModelError::EntityAlreadyExists).into());
    }

    let result: Product = diesel::insert_into(products::table)
        .values(new_product)
        .returning(Product::as_returning())
        .get_result(&mut conn)
        .await?;

    Ok(Json(result))
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_product))
        .route("/create", post(create_product))
        .with_state(state.clone())
}
