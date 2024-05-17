use serde::{Deserialize, Serialize};
use snowflake::SnowflakeGenerator;
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

use super::{assignments::Assignment, strings};

#[derive(Serialize, Deserialize, Default, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub display_name: String,
    pub comment: Option<String>,
    pub in_service: bool,
    #[sqlx(skip)]
    pub assignment: Option<Assignment>,
}

pub async fn create_resource(
    pool: &Pool<Sqlite>,
    display_name: &str,
    comment: Option<String>,
) -> Result<Resource, sqlx::Error> {
    let id: String = SnowflakeGenerator::new(0, 0).generate().to_string();

    let resource = sqlx::query_as::<_, Resource>(&strings::CREATE_RESOURCE)
        .bind(&id)
        .bind(display_name)
        .bind(comment)
        .fetch_one(pool)
        .await?;

    Ok(resource)
}

pub async fn set_in_service(
    pool: &Pool<Sqlite>,
    id: &str,
    in_service: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(&strings::UPDATE_RESOURCE_IN_SERVICE)
        .bind(in_service)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Resource>, sqlx::Error> {
    let resources = sqlx::query(&strings::GET_RESOURCES)
        .map(|row: SqliteRow| Resource {
            id: row.get("resource_id"),
            display_name: row.get("display_name"),
            in_service: row.get("in_service"),
            comment: row.get("comment"),
            assignment: match row.get::<Option<String>, _>("aa_id") {
                Some(_) => Some(Assignment {
                    id: row.get("aa_id"),
                    job_id: row.get("job_id"),
                    resource_id: row.get("resource_id"),
                    assigned_at: row.get("assigned_at"),
                    removed_at: row.get("removed_at"),
                    assigned_by: row.get("assigned_by"),
                    removed_by: row.get("removed_by"),
                }),
                None => None,
            },
        })
        .fetch_all(pool)
        .await?;
    Ok(resources)
}
