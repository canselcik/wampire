extern crate eventual;
extern crate wampire;
use eventual::Async;
use std::{io, env};
use std::sync::{Arc, Mutex};
use wampire::client::{Client, Connection, Subscription};
use wampire::{MatchingPolicy, Value, URI};
use std::env::args;

#[macro_use]
extern crate log;
extern crate env_logger;


fn subscribe(
    client: &mut Client,
    subscriptions: &mut Arc<Mutex<Vec<Subscription>>>,
    topic: String,
    policy: MatchingPolicy,
) {
    let subscriptions = Arc::clone(subscriptions);
    client
        .subscribe_with_pattern(
            URI::new(&topic),
            Box::new(move |args, kwargs| {
                println!(
                    "Received message on topic {} with args {:?} and kwargs {:?}",
                    topic, args, kwargs
                );
            }),
            policy,
        )
        .unwrap()
        .and_then(move |subscription| {
            println!("Subscribed to topic {}", subscription.topic.uri);
            subscriptions.lock().unwrap().push(subscription);
            Ok(())
        })
        .await()
        .unwrap();
}

fn unsubscribe(
    client: &mut Client,
    subscriptions: &mut Arc<Mutex<Vec<Subscription>>>,
    args: &[String],
) {
    if args.len() > 1 {
        println!("Too many arguments to subscribe.  Ignoring");
    } else if args.is_empty() {
        println!("Please specify the topic to subscribe to");
        return;
    }
    match args[0].parse::<usize>() {
        Ok(i) => {
            let mut subscriptions = subscriptions.lock().unwrap();
            if i >= subscriptions.len() {
                println!("Invalid subscription index: {}", i);
                return;
            }
            let subscription = subscriptions.remove(i);
            let topic = subscription.topic.uri.clone();
            client.unsubscribe(subscription)
                .unwrap()
                .and_then(move |()| {
                    println!("Successfully unsubscribed from {}", topic);
                    Ok(())
                })
                .await()
                .unwrap();
        }
        Err(_) => {
            println!("Invalid subscription index: {}", args[0]);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
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
