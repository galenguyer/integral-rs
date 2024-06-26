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
    pub current_assignment: Option<Assignment>,
    #[sqlx(skip)]
    pub location: Option<GeocodeResponse>,
}

#[derive(Serialize, Deserialize, Default, Debug, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLocation {
    #[serde(skip)]
    pub resource_id: String,
    pub at_time: i64,
    pub latitude: String,
    pub longitude: String,
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
    resource_id: &str,
    in_service: bool,
    assigned_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(&strings::UPDATE_RESOURCE_IN_SERVICE)
        .bind(in_service)
        .bind(resource_id)
        .execute(pool)
        .await?;

    if !in_service {
        sqlx::query(&strings::UPDATE_ASSIGNMENTS_RESOURCE_OOS)
            .bind(assigned_by)
            .bind(resource_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize, FromRow, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeocodeResponse {
    pub lat: String,
    pub lon: String,
    pub distance: f64,
    pub address: RadarAddress,
}
#[derive(Serialize, Deserialize, FromRow, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RadarAddress {
    address_label: Option<String>,
    city: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    county: Option<String>,
    formatted_address: Option<String>,
    latitude: Option<f64>,
    layer: Option<String>,
    longitude: Option<f64>,
    number: Option<String>,
    postal_code: Option<String>,
    state: Option<String>,
    state_code: Option<String>,
    street: Option<String>,
}

pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Resource>, sqlx::Error> {
    let locations = sqlx::query_as::<_, ResourceLocation>(
        "SELECT * FROM resource_locations GROUP BY resource_id HAVING at_time = MAX(at_time)",
    )
    .fetch_all(pool)
    .await
    .unwrap();

    let resp = ureq::post("http://127.0.0.1:8081/api/v0/geocode/reverse/bulk")
        .send_json(ureq::json!(locations
            .iter()
            .map(|loc| ureq::json!({"lat": loc.latitude, "lon": loc.longitude}))
            .collect::<Vec<_>>()))
        .unwrap()
        .into_json::<Vec<GeocodeResponse>>()
        .unwrap();

    let resources = sqlx::query(&strings::GET_RESOURCES)
        .map(|row: SqliteRow| Resource {
            id: row.get("resource_id"),
            display_name: row.get("display_name"),
            in_service: row.get("in_service"),
            comment: row.get("comment"),
            current_assignment: match row.get::<Option<String>, _>("aa_id") {
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
            location: resp
                .iter()
                .find(|gcr| {
                    let a = locations
                        .iter()
                        .find(|l| l.resource_id == row.get::<String, _>("resource_id"));
                    a.is_some_and(|b| b.latitude == gcr.lat && b.longitude == gcr.lon)
                })
                .cloned(),
        })
        .fetch_all(pool)
        .await?;
    Ok(resources)
}


pub async fn set_location(
    pool: &Pool<Sqlite>,
    resource_id: &str,
    latitude: &str,
    longitude: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO resource_locations(resource_id,latitude,longitude) VALUES (?, ?, ?)")
        .bind(resource_id)
        .bind(latitude)
        .bind(longitude)
        .execute(pool)
        .await?;

    Ok(())
}
