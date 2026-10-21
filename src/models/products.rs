use crate::schema::products;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(HasQuery, Serialize)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = products)]
pub struct UpdatePrice {
    pub id: i32,
    pub price: f64,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = products)]
pub struct UpdateName {
    pub id: i32,
    pub name: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = products)]
pub struct UpdateDescription {
    pub id: i32,
    pub description: String,
}
