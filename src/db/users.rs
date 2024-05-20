use serde::{Deserialize, Serialize};
use snowflake::SnowflakeGenerator;
use sqlx::{FromRow, Pool, Sqlite};

use crate::db::strings;

#[cfg(debug_assertions)]
const BCRYPT_COST: u32 = 8;
#[cfg(not(debug_assertions))]
const BCRYPT_COST: u32 = 14;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: i64,
    pub admin: bool,
    pub enabled: bool,
}

pub async fn get_user(pool: &Pool<Sqlite>, id: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(&strings::GET_USER_BY_ID)
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(user)
}

pub async fn get_user_by_email(pool: &Pool<Sqlite>, email: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(&strings::GET_USER_BY_EMAIL)
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(user)
}

pub async fn create_user(
    pool: &Pool<Sqlite>,
    email: &str,
    password: &str,
    display_name: &str,
) -> Result<User, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let id = SnowflakeGenerator::new(0, 0).generate().to_string();

    let user = sqlx::query_as::<_, User>(&strings::CREATE_USER)
        .bind(id)
        .bind(email)
        .bind(bcrypt::hash(password, BCRYPT_COST).unwrap())
        .bind(display_name)
        .fetch_one(&mut *transaction)
        .await?;

    transaction.commit().await?;
    Ok(user)
}
