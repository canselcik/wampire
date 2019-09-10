extern crate env_logger;
extern crate eventual;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate wampire;

use std::{env, io};
use std::env::args;
use std::sync::{Arc, Mutex};

use eventual::Async;
use serde::Serialize;
use serde_json::Serializer;

use wampire::{MatchingPolicy, URI, Value};
use wampire::client::{Client, Connection, Subscription};


fn subscribe(
    client: &mut Client,
    subscriptions: &mut Arc<Mutex<Vec<Subscription>>>,
    topic: String,
    policy: MatchingPolicy,
) {
    let subscriptions = Arc::clone(subscriptions);
    client.subscribe_with_pattern(
            URI::new(&topic),
            Box::new(move |args, kwargs| {
//                println!(
//                    "{} => ( {:?} )-( {:?} )",
//                    topic, args, kwargs
//                );

                let val = args.get(0).unwrap();
                let snapshot = match val {
                    Value::String(ref s) => s.clone(),
                    _ => String::new()
                };
                println!("{}", snapshot);
            }),
            policy,
        )
        .unwrap()
        .and_then(move |subscription| {
            info!("Subscribed to topic {}", subscription.topic.uri);
            subscriptions.lock().unwrap().push(subscription);
            Ok(())
        })
        .await()
        .unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    info!("{:?}", args);
    let mut url: String = "ws://127.0.0.1:9000/ws".to_string();
    if args.len() > 1 {
        url = args[1].to_string();
    }
    let mut realm: String = "test.it".to_string();
    if args.len() > 2 {
        realm = args[2].to_string();
    }

    env_logger::init();
    let connection = Connection::new(url.as_str(), realm.as_str());
    info!("Connecting");
    let mut client = connection.connect().unwrap();
    info!("Connected");

    let mut subscriptions = Arc::new(Mutex::new(Vec::new()));
    subscribe(&mut client, &mut subscriptions, "#".to_string(), MatchingPolicy::Mqtt);
    loop {}
}
