use geo::{Coord, Point};

pub const SCL_OUTAGE_LIST_URL: &str = "https://utilisocial.io/datacapable/v2/p/scl/map/events";
pub const SCL_LAST_UPDATE_URL: &str = "https://utilisocial.io/datacapable/v2/p/scl/map/stats";
pub const SCL_POLLING_INTERVAL_IN_MINS: u64 = 5;

/** 
 * TODO:
 * Replace with the coordinates of the location in Seattle we're monitoring power for,
 * in WGS84 "projection" - yes, degrees
*/
pub const LOCATION: Point = Point(Coord {x: -122.3507297, y: 47.6205405});
pub const RADIUS: f64 = 0.000125; // technically, we'll be using a square, not a circle, as our "radius"

/** 
 * TODO:
 * Enter the BOT TOKEN and CHAT ID provided by Telegram for sending messages
 * using your Telegram bot.
*/
pub const TELEGRAM_BOT_TOKEN: &str = "1234567890:AbCdEfGhIjKlMnOpQrStUvWxYz";
pub const TELEGRAM_CHAT_ID: &str = "-1234512345123";

pub enum PowerStatus {
    OFFLINE,
    ONLINE
}