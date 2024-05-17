use crate::db::strings;
use serde::{Deserialize, Serialize};
use snowflake::SnowflakeGenerator;
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Serialize, Deserialize, FromRow, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub synopsis: String,
    pub location: Option<String>,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    pub created_by: String,
    pub closed_by: Option<String>,
    #[sqlx(skip)]
    pub comments: Vec<Comment>,
}

#[derive(Default, Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub job_id: String,
    pub comment: String,
    pub created_at: i64,
    pub created_by: String,
}

pub async fn get_all_jobs(pool: &Pool<Sqlite>) -> Result<Vec<Job>, sqlx::Error> {
    let jobs = sqlx::query_as::<_, Job>(&strings::GET_ALL_JOBS)
        .fetch_all(pool)
        .await?;
    Ok(jobs)
}

pub async fn get_job_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Job>, sqlx::Error> {
    let job = sqlx::query_as::<_, Job>(&strings::GET_JOB_BY_ID)
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match job {
        Some(mut job) => {
            let comments = sqlx::query_as::<_, Comment>(&strings::GET_COMMENTS_FOR_JOB)
                .bind(id)
                .fetch_all(pool)
                .await?;
            job.comments = comments;
            Ok(Some(job))
        }
        None => Ok(None),
    }
}

pub async fn create_job(
    pool: &Pool<Sqlite>,
    synopsis: &str,
    location: Option<String>,
    created_by: &str,
) -> Result<Job, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let id: String = SnowflakeGenerator::new(0, 0).generate().to_string();

    let user = sqlx::query_as::<_, Job>(&strings::CREATE_JOB)
        .bind(id)
        .bind(synopsis)
        .bind(location)
        .bind(created_by)
        .fetch_one(&mut *transaction)
        .await?;

    transaction.commit().await?;
    Ok(user)
}

pub async fn add_comment(
    pool: &Pool<Sqlite>,
    job_id: &str,
    comment: &str,
    created_by: &str,
) -> Result<Comment, sqlx::Error> {
    let id: String = SnowflakeGenerator::new(0, 0).generate().to_string();

    let new_comment = sqlx::query_as::<_, Comment>(&strings::ADD_COMMENT)
        .bind(&id)
        .bind(job_id)
        .bind(comment)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

    Ok(new_comment)
}

pub async fn close_job(
    pool: &Pool<Sqlite>,
    job_id: &str,
    closed_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(&strings::CLOSE_JOB)
        .bind(closed_by)
        .bind(job_id)
        .execute(pool)
        .await?;
    sqlx::query(&strings::CLOSE_ASSIGNMENTS_FOR_JOB)
        .bind(closed_by)
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}
