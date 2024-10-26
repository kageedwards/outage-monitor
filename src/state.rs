use parking_lot::Mutex;
use teloxide::prelude::*;
use crate::{TELEGRAM_BOT_TOKEN, Outage, PowerStatus, dbg_println};

pub struct ApplicationState {
    pub status: Mutex<PowerStatus>,
    pub last_update: Mutex<i64>,
    pub outages: Mutex<Vec<Outage>>,
    pub telegram: Mutex<Bot>
}

impl ApplicationState {
    pub fn new() -> ApplicationState {
        ApplicationState {
            status: Mutex::new(PowerStatus::ONLINE),
            last_update: Mutex::new(0i64),
            outages: Mutex::new(vec![]),
            telegram: Mutex::new(Bot::new(TELEGRAM_BOT_TOKEN))
        }
    }

    pub fn is_new_data_available(&self, timestamp: i64) -> bool {
        let mut last_update = self.last_update.lock();
        let mut is_available = false;

        if timestamp > *last_update {
            dbg_println!("New data is available.");
            *last_update = timestamp;

            is_available = true;
        }

        is_available
    }

    pub fn update_data(&self, mut items: Vec<Outage>) {
        let mut outages = self.outages.lock();

        outages.clear();
        outages.append(&mut items);
    }

    pub fn get_data(&self) -> Mutex<Vec<Outage>> {
        let outages = self.outages.lock();

        Mutex::new(outages.to_vec())
    }

    pub fn send_telegram(&self) -> Bot {
        let bot = self.telegram.lock();
        
        //Mutex::new(bot.clone())
        bot.clone()
    }
}