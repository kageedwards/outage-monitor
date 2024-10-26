mod constants;
mod helpers;
mod response_structs;
mod state;

use parking_lot::Mutex;
use tokio::time::{Duration, MissedTickBehavior};
use ansi_term::Colour::Red as ErrorColor;
use teloxide::prelude::Requester;
use reqwest::Response;
use geo::{point, Coord, Intersects, LineString, Polygon, Rect};

/**
 * Adjust location coordinates and Telegram credentials in constants.rs
 */

use crate::constants::*;
use crate::helpers::Result;
use crate::response_structs::{Outage, StatsResponse};
use crate::state::ApplicationState;

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
                dbg_println!("{}", ErrorColor.paint("Timestamp update has failed."));
                dbg_println!("{:?}", err.to_string());
                false
            }
        };

        if is_data_available {
            match fetch_outages().await {
                Ok(items) => state.update_data(items),
                Err(err) => {
                    dbg_println!("{}", ErrorColor.paint("Outage data request failed."));
                    dbg_println!("{:?}", err.to_string());
                }
            }
        }
        
        match check_power_status(state.get_data()).await {
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
                dbg_println!("{}", ErrorColor.paint("Failed to process outage data."));
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

    let outages_data = fetch::<Vec<Outage>>(SCL_OUTAGE_LIST_URL).await?;

    Ok(outages_data)
}

/**
 *  Requests the `SCL_LAST_UPDATE_URL` endpoint to determine if our current outage data is stale
 *   
 *  @return     Result<i64> : Result-wrapped UNIX timestamp as a signed 64-bit integer, designating the time of the last update on the server
 *  @propagates Request errors, JSON deserialization errors, Parse errors
*/
async fn fetch_last_update() -> Result<i64> {
    dbg_println!("Checking for updates...");

    let stats_data = fetch::<StatsResponse>(SCL_LAST_UPDATE_URL).await?;

    let last_update = stats_data.last_updated_time.parse::<i64>()?;

    Ok(last_update)
}

/**
 *  Sends an HTTP request to a specified endpoint and returns an attempted
 *  deserialization of a JSON response into the generic type T
 *   
 *  @typeParam  T : the type into which the JSON response should be deserialized (must derive Deserialize)
 *  @param      &str : url : the URL of the endpoint being requested
 * 
 *  @return     Result<T> : Result-wrapped object of type T
 *  @propagates Request errors, JSON deserialization errors
*/
async fn fetch<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let response: Response = reqwest::get(url).await?;
    
    // This method is more verbose, but `serde_json` propagates more
    // detailed error information than the `response.json()` method does.
    let data = response.bytes().await?;

    Ok(serde_json::from_slice::<T>(&data)?)
}

/**
 *  Parses the polygonal areas of each affected power outage area and returns whether power is online or not
 *
 *  @param  Mutex<Vec<Outage>> : outages : a mutually exclusive reference to a list of power outages (@see `struct Outage`)
 *   
 *  @return Result<bool> : Result-wrapped boolean designating whether the power is currently ONLINE
*/
async fn check_power_status(outages: Mutex<Vec<Outage>>) -> Result<bool> {
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
