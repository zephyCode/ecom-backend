use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, FromRow};

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Serialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub img_url: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock_quantity: i32,
    pub category: Option<String>,
    pub img_url: Option<String>,
}

#[post("/product")]
async fn add_product(
    product: web::Json<NewProduct>,
    pool: web::Data<MySqlPool>
) -> impl Responder {

    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        r#"
        INSERT INTO Products 
        (name, description, price, stock_quantity, category, img_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        product.name,
        product.description,
        product.price,
        product.stock_quantity,
        product.category,
        product.img_url,
        now,
        now
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json("Product added successfully."),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to add product.".to_string()
            })
        }
    }
}

#[get("/products")]
async fn get_all_products(pool: web::Data<MySqlPool>) -> impl Responder {
    let result = sqlx::query_as!(
        Product,
        r#"
        SELECT
            id,
            name,
            description,
            price,
            stock_quantity,
            category,
            img_url,
            created_at,
            updated_at
        FROM Products
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => {
            eprintln!("DB error fetching products: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to fetch products!".to_string()
            })
        }
    }
}

#[get("/product/{id}")]
async fn get_product_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query_as!(
        Product,
        r#"
        SELECT 
            id,
            name,
            description,
            price,
            stock_quantity,
            category,
            img_url,
            created_at,
            updated_at
        FROM Products
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            message: "Product not found!".to_string()
        }),
        Err(e) => {
            eprintln!("DB error fetching product: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to fetch product!".to_string()
            })
        }
    }
}

#[put("/product/{id}")]
async fn update_product_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    product: web::Json<NewProduct>,
) -> impl Responder {
    let id = path.into_inner();
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        r#"
        UPDATE Products
        SET
            name = ?,
            description = ?,
            price = ?,
            stock_quantity = ?,
            category = ?,
            img_url = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        product.name,
        product.description,
        product.price,
        product.stock_quantity,
        product.category,
        product.img_url,
        now,
        id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "Product not found!".to_string(),
                });
            }

            HttpResponse::Ok().json("Product updated successfully.")
        }
        Err(e) => {
            eprintln!("DB error updating product: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to update product!".to_string(),
            })
        }
    }
}

#[delete("/product/{id}")]
async fn delete_product_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query!(
        r#"
            DELETE FROM Products WHERE id = ?
        "#,
        id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "Product not found!".to_string(),
                });
            }

            HttpResponse::Ok().json("Product deleted successfully.")
        }
        Err(e) => {
            eprintln!("DB error updating product: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to delete the product!".to_string(),
            })
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(add_product);
    cfg.service(get_all_products);
    cfg.service(get_product_by_id);
    cfg.service(update_product_by_id);
    cfg.service(delete_product_by_id);
}
