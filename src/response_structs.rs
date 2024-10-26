use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Outage {
    pub id: i32,
    #[serde(rename = "type")]
    pub outage_type: Option<String>,
    // #[serde(rename = "startTime")]
    // start_time: u64,
    // #[serde(rename = "lastUpdatedTime")]
    // last_updated_time: u64,
    // #[serde(rename = "etrTime")]
    // etr_time: u64,
    #[serde(default, rename = "numPeople")]
    pub people_affected: Option<i32>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub cause: Option<String>,
    pub polygons: OutagePolygon
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutagePolygon {
    #[serde(rename= "spatialReference")]
    pub spatial_reference: Option<SpatialReference>,
    #[serde(rename = "rings")]
    pub areas: Vec<Vec<Vec<f64>>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpatialReference {
    #[serde(rename = "latestWkid")]
    pub latest_wkid: i32,
    pub wkid: i32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatsResponse {
    #[serde(rename = "lastUpdatedTime")]
    pub last_updated_time: String
}