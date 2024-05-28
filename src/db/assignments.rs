use serde::{Deserialize, Serialize};
use snowflake::SnowflakeGenerator;
use sqlx::{FromRow, Pool, Sqlite};

use super::strings;

#[derive(Serialize, Deserialize, Default, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
    pub id: String,
    pub resource_id: String,
    pub job_id: String,
    pub assigned_at: i64,
    pub removed_at: Option<i64>,
    pub assigned_by: String,
    pub removed_by: Option<String>,
}

pub async fn get_active_assignments(pool: &Pool<Sqlite>) -> Result<Vec<Assignment>, sqlx::Error> {
    let assignments = sqlx::query_as::<_, Assignment>(&strings::GET_ACTIVE_ASSIGNMENTS)
        .fetch_all(pool)
        .await?;
    Ok(assignments)
}

pub async fn get_assignments_for_job(pool: &Pool<Sqlite>, job_id: &str) -> Result<Vec<Assignment>, sqlx::Error> {
    let assignments = sqlx::query_as::<_, Assignment>(&strings::GET_ASSIGNMENTS_BY_JOBID)
        .bind(&job_id)
        .fetch_all(pool)
        .await?;
    Ok(assignments)
}

pub async fn assign(
    pool: &Pool<Sqlite>,
    job_id: &str,
    resource_id: &str,
    assigned_by: &str,
) -> Result<Assignment, sqlx::Error> {
    let id: String = SnowflakeGenerator::new(0, 0).generate().to_string();

    let assignment = sqlx::query_as::<_, Assignment>(&strings::CREATE_ASSIGNMENT)
        .bind(&id)
        .bind(job_id)
        .bind(resource_id)
        .bind(assigned_by)
        .fetch_one(pool)
        .await?;
    Ok(assignment)
}

pub async fn unassign(
    pool: &Pool<Sqlite>,
    assignment_id: &str,
    assigned_by: &str,
) -> Result<(), sqlx::Error> {
    let _assignment = sqlx::query(&strings::REMOVE_ASSIGNMENT)
        .bind(assigned_by)
        .bind(assignment_id)
        .execute(pool)
        .await?;
    Ok(())
}
