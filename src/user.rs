use actix_web::{HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool};

#[derive(Deserialize, Serialize)]
struct NewUser {
    name: String,
    email: String,
    password: String
}

#[derive(Deserialize, Serialize)]
struct ExistingUser {
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct UserResponse {
    id: i32,
    email: String,
    name: String
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String
}

pub async fn get_all_users(pool: &MySqlPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!("SELECT email FROM Users")
        .fetch_all(pool)
        .await?;

    let users = rows
        .into_iter()
        .map(|row| row.email)
        .collect();

    Ok(users)
}

pub async  fn get_user_by_id(pool: &MySqlPool, id: i32) -> Result<Option<UserResponse>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT * FROM Users WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| UserResponse {
        id: r.id,
        name: r.name,
        email: r.email
    }))
}


#[post("/signup")]
async fn sign_up(
    user: web::Json<NewUser>, 
    pool: web::Data<MySqlPool>
) -> impl Responder {

    match get_all_users(pool.get_ref()).await {
        Ok(all_users) => {
            for existing_email in all_users {
                if existing_email == user.email {
                    return HttpResponse::BadRequest().json(ErrorResponse {
                        message: "User with this email already exists!".to_string(),
                    });
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch users: {:?}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to check existing users.".to_string(),
            });
        }
    }

    if user.password.trim().len() < 8 {
        return 
        HttpResponse::BadRequest()
            .json(ErrorResponse {
                message: "Password cannot be less than 8 characters!".to_string()
            });
    }

    let result = sqlx::query!(
        "INSERT INTO Users (name, email, password) VALUES (?, ?, ?)",
        user.name,
        user.email,
        user.password
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User added successfully!"),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to add user!")
        }
    }
}

#[post("/login")]
async fn login(
    user: web::Json<ExistingUser>,
    pool: web::Data<MySqlPool>
) -> impl Responder {
    let row = sqlx::query!(
        "Select * from Users where email = ?",
        user.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(Some(record)) => {

            let stored_email = record.email;
            let stored_password = record.password;

            if stored_email == user.email && stored_password == user.password {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Logged in successfully."
                }))
            }
            else {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    message: "Incorrect passsword!".to_string(),
                })
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ErrorResponse {
                message: "No user found!".to_string(),
            })
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Database error occured!".to_string(),
            })
        }
    }
}

#[get("/users/{id}")]
async fn get_user(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>
) -> impl Responder {
    
    let id = path.into_inner();

    match crate::user::get_user_by_id(pool.get_ref(), id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "No user found for provided ID!"
        })),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Database error!"
            }))
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_up);
    cfg.service(login);
    cfg.service(get_user);
}