// really lazy program to simulate requests to the server
// Edit constant directly for

use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::process::id;
use std::thread::sleep;

enum Action {
    ListRandomTable,
    CreateAndRemove,
}

#[derive(Deserialize, Serialize)]
struct OrderCreateRequest {
    table_number: u8,
    item_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct Order {
    id: i32,
}

fn main() {
    // use first cli argument as url
    let url = "http://localhost:8081/orders/";
    let max_thread = 50;
    let wait_time = 25;
    let table_count = 200;

    for thread_id in 0..max_thread {
        std::thread::spawn(move || {
            loop {
                sleep(std::time::Duration::from_millis(wait_time));

                // Chose an action at Random
                let action: Action = match rand::random::<u8>() % 2 {
                    0 => Action::ListRandomTable,
                    1 => Action::CreateAndRemove,
                    _ => Action::ListRandomTable,
                };

                match action {
                    Action::ListRandomTable => {
                        let response = reqwest::blocking::get(format!(
                            "{}?table_number={}",
                            url,
                            rand::random::<u8>() % table_count
                        ));

                        println!("{} - {:?}", thread_id, response)
                    }
                    Action::CreateAndRemove => {
                        let client = reqwest::blocking::Client::new();
                        let response = client
                            .post(url)
                            .json(&OrderCreateRequest {
                                table_number: rand::random::<u8>() % table_count,
                                item_name: "potato".to_string(),
                            })
                            .send();

                        println!("{} - {:?}", thread_id, response);

                        let order: Order = response.unwrap().json().unwrap();
                        let response = client.delete(format!("{}{}", url, order.id)).send();

                        println!("{} - {:?}", thread_id, response);
                    }
                }
            }
        });

        // Leave program hanging...
        sleep(std::time::Duration::from_secs(7 * 24 * 60 * 60));
    }
}
