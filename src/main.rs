use parking_lot::Mutex;
use reqwest::Response;
use geo::{point, Coord, Intersects, LineString, Point, Polygon, Rect};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, MissedTickBehavior};
use ansi_term::Colour::Red;
use teloxide::prelude::*;

const SCL_OUTAGE_LIST_URL: &str = "https://utilisocial.io/datacapable/v2/p/scl/map/events";
const SCL_LAST_UPDATE_URL: &str = "https://utilisocial.io/datacapable/v2/p/scl/map/lastUpdated";
const SCL_POLLING_INTERVAL_IN_MINS: u64 = 5;

/** 
 * TODO:
 * Replace with the coordinates of the location in Seattle we're monitoring power for,
 * in WGS84 "projection" - yes, degrees
*/
const LOCATION: Point = Point(Coord {x: -122.3507297, y: 47.6205405});
const RADIUS: f64 = 0.000125; // technically, we'll be using a square, not a circle, as our "radius"

/** 
 * TODO:
 * Enter the BOT TOKEN and CHAT ID provided by Telegram for sending messages
 * using your Telegram bot.
*/
const TELEGRAM_BOT_TOKEN: &str = "1234567890:AbCdEfGhIjKlMnOpQrStUvWxYz";
const TELEGRAM_CHAT_ID: &str = "-1234512345123";


// A macro to gracefully ignore println! statements if we aren't in debug mode.
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

// A simple type alias to simplify error propagation from disparate sources (DRY).
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


/* JSON Response Body Structures */

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Outage {
    id: i32,
    #[serde(rename = "type")]
    outage_type: Option<String>,
    // #[serde(rename = "startTime")]
    // start_time: u64,
    // #[serde(rename = "lastUpdatedTime")]
    // last_updated_time: u64,
    // #[serde(rename = "etrTime")]
    // etr_time: u64,
    #[serde(default, rename = "numPeople")]
    people_affected: Option<i32>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    cause: Option<String>,
    polygons: OutagePolygon
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OutagePolygon {
    #[serde(rename= "spatialReference")]
    spatial_reference: Option<SpatialReference>,
    #[serde(rename = "rings")]
    areas: Vec<Vec<Vec<f64>>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpatialReference {
    #[serde(rename = "latestWkid")]
    latest_wkid: i32,
    wkid: i32
}

/* END: JSON Response Body Structures */

enum PowerStatus {
    OFFLINE,
    ONLINE
}

struct ApplicationState {
    status: Mutex<PowerStatus>,
    last_update: Mutex<i64>,
    outages: Mutex<Vec<Outage>>,
    telegram: Mutex<Bot>
}

impl ApplicationState {
    fn new() -> ApplicationState {
        ApplicationState {
            status: Mutex::new(PowerStatus::ONLINE),
            last_update: Mutex::new(0i64),
            outages: Mutex::new(vec![]),
            telegram: Mutex::new(Bot::new(TELEGRAM_BOT_TOKEN))
        }
    }

    fn is_new_data_available(&self, timestamp: i64) -> bool {
        let mut last_update = self.last_update.lock();
        let mut is_available = false;

        if timestamp > *last_update {
            dbg_println!("New data is available.");
            *last_update = timestamp;

            is_available = true;
        }

        is_available
    }

    fn update_data(&self, mut items: Vec<Outage>) {
        let mut outages = self.outages.lock();

        outages.clear();
        outages.append(&mut items);
    }

    fn get_data(&self) -> Mutex<Vec<Outage>> {
        let outages = self.outages.lock();

        Mutex::new(outages.to_vec())
    }

    fn send_telegram(&self) -> Bot {
        let bot = self.telegram.lock();
        
        //Mutex::new(bot.clone())
        bot.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(SCL_POLLING_INTERVAL_IN_MINS * 60));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let state: ApplicationState = ApplicationState::new();

    println!("╔══════════════════════════════════════╗");
    println!("║  Seattle City Light  Status Monitor  ║");
    println!("╚══════════════════════════════════════╝");

    loop {
        interval.tick().await;

        let is_data_available = match fetch_last_update().await {
            Ok(timestamp) => state.is_new_data_available(timestamp),
            Err(err) => {
                dbg_println!("{}", Red.paint("Timestamp update has failed."));
                dbg_println!("{:?}", err.to_string());
                false
            }
        };

        if is_data_available {
            match fetch_outages().await {
                Ok(items) => state.update_data(items),
                Err(err) => {
                    dbg_println!("{}", Red.paint("Outage data request failed."));
                    dbg_println!("{:?}", err.to_string());
                }
            }
        }
        
        match fetch_power_status(state.get_data()).await {
            Ok(is_now_online) => {
                let mut stored_online_status = state.status.lock();

                match *stored_online_status {
                    PowerStatus::ONLINE if !is_now_online => {

                            let bot = state.send_telegram();
                            bot.send_message(TELEGRAM_CHAT_ID.to_string(), "POWER OUTAGE affecting the location.").await?;

                            dbg_println!("There is a POWER OUTAGE at the location.");

                            *stored_online_status = PowerStatus::OFFLINE;
                    },
                    PowerStatus::OFFLINE if is_now_online => {

                            let bot = state.send_telegram();
                            bot.send_message(TELEGRAM_CHAT_ID.to_string(), "Power is ONLINE at location.").await?;

                            dbg_println!("Power is back ONLINE at the location.");

                            *stored_online_status = PowerStatus::ONLINE;
                    },
                    PowerStatus::ONLINE => {
                        dbg_println!("No outages present at the location.");
                    },
                    PowerStatus::OFFLINE => {
                        dbg_println!("Power remains DOWN at the location.");
                    }
                }
            },
            Err(err) => {
                dbg_println!("{}", Red.paint("Failed to process outage data."));
                dbg_println!("{:?}", err.to_string());
            }
        }

        dbg_println!("════════════════════════════════════════");

    }

}

/**
 *  Requests the latest outage data from the `SCL_OUTAGE_LIST_URL` endpoint, and returns a deserialized list. 
 *   
 *  @return     Result<Vec<Outage>> : Result-wrapped list of current power outages, deserialized into `Outage` instances
 *  @propagates Request errors, JSON deserialization errors
*/
async fn fetch_outages() -> Result<Vec<Outage>> {
    dbg_println!("Fetching new outage data...");

    let response: Response = reqwest::get(SCL_OUTAGE_LIST_URL).await?;
    
    // This method is more verbose, but `serde_json` propagates more
    // detailed error information than the `response.json()` method does.
    let outages_data = response.bytes().await?;

    Ok(serde_json::from_slice::<Vec<Outage>>(&outages_data)?)
}

/**
 *  Requests the `SCL_LAST_UPDATE_URL` endpoint to determine if our current outage data is stale
 *   
 *  @return     Result<i64> : Result-wrapped UNIX timestamp as a signed 64-bit integer, designating the time of the last update on the server
 *  @propagates Request errors, Parse errors
*/
async fn fetch_last_update() -> Result<i64> {
    dbg_println!("Checking for updates...");

    let last_update_response: String = reqwest::get(SCL_LAST_UPDATE_URL)
        .await?
        .text()
        .await?;

    let last_update = last_update_response.parse::<i64>()?;

    Ok(last_update)
}

/**
 *  Parses the polygonal areas of each affected power outage area and returns whether power is online or not
 *
 *  @param  Mutex<Vec<Outage>> : outages : a mutually exclusive reference to a list of power outages (@see `struct Outage`)
 *   
 *  @return Result<bool> : Result-wrapped boolean designating whether the power is currently ONLINE
*/
async fn fetch_power_status(outages: Mutex<Vec<Outage>>) -> Result<bool> {
    dbg_println!("Processing outage data...");

    let known_outages = outages.lock();
        
    let mut polys: Vec<Polygon> = vec![];

    // Refactor points (Vec[2]<f64> to Coord{x,y}) and collect Polygons
    for outage in known_outages.iter() {
        let outage = outage.clone();

        let exterior = match outage.polygons.areas.get(0) {
            Some(area_data) => area_data.clone(),
            None => vec![]
        };

        let mut items: Vec<Coord> = vec![]; 
        for vect in exterior {
            let point = Coord { x: vect[0], y: vect[1]};
            items.push(point);
        }

        polys.push(Polygon::new(LineString::new(items), vec![]));
    }

    #[cfg(debug_assertions)] {
        let outage_count: usize = polys.len();

        if outage_count == 1 {
            // dbg_println! effectively
            println!("➤ {0} reported outage in Seattle", outage_count);
        } else {
            // dbg_println! effectively
            println!("➤ {0} reported outages in Seattle", outage_count);
        }
    }   

    let mut outage_present = false;

    // Calculate a square perimeter RADIUS -/+ around the center point LOCATION
    let loc = point!(x:LOCATION.x(), y: LOCATION.y());
    let rect = Rect::new(Coord {x: loc.x() - RADIUS, y: loc.y() - RADIUS}, Coord {x: loc.x() + RADIUS, y: loc.y() + RADIUS});

    for poly in polys.iter() {
        let poly = poly.clone();

        // Compare it with each outage area
        outage_present = poly.intersects(&rect);

        if outage_present {
            break;
        }
    }

    Ok(!outage_present)
}