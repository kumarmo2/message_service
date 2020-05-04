mod business;
mod dtos;
mod utils;
use crate::business::handle_message_event as handle_message_event_bl;
use chat_common_types::events::MessageEvent;
use lapin::{message::Delivery, options::*, types::FieldTable, Channel, Consumer};
use manager::{ChannelManager, PooledConnection, RabbitMqManager};
use num_cpus;
use serde_json::from_str;
use smol::{self, block_on, Task};
use std::time::Instant;
use utils::create_client;

fn main() {
    println!("hello world");
    // TODO: retrieve from the config based on environment.
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    // TODO: update the RabbitMqManager to accept the max number of connections and channels.
    let rabbit = RabbitMqManager::new(addr);
    let http_client = create_client();
    let cpus = num_cpus::get().max(1);
    for _ in 0..cpus {
        std::thread::spawn(move || smol::run(futures::future::pending::<()>()));
    }

    block_on(start_consumer(rabbit, "messages", http_client));
}

async fn start_consumer(rabbit: RabbitMqManager, queue_name: &str, http_client: reqwest::Client) {
    let mut channel: PooledConnection<ChannelManager>;
    println!("will get");
    loop {
        match rabbit.get_channel_pool().get() {
            Ok(ch) => {
                channel = ch;
            }
            Err(reason) => {
                println!("could not get channel, reason: {:?}", reason);
                return;
            }
        }
        println!("got channel");
        let consumer: Consumer = channel
            .basic_consume(
                queue_name,
                "message_service",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
        // TODO: need to check if cloning of htt_client can be reduced. and what kind of
        // performance impact does the cloning has here.
        process_messages(consumer, channel, http_client.clone()).await;
    }
}

async fn process_messages(
    consumer: Consumer,
    channel: PooledConnection<ChannelManager>,
    http_client: reqwest::Client,
) {
    println!("start of consumer processing");
    let now = Instant::now();
    for delivery in consumer {
        println!(
            "message received, thread_id: {:?}",
            std::thread::current().id()
        );
        if let Ok(delivery) = delivery {
            // TODO: Read more about rabbitmq's channel. check if there is scope for optimizations.
            Task::spawn(handle_messasge_event(
                delivery,
                channel.clone(),
                http_client.clone(),
            ))
            .detach()
        }
    }
    println!(
        "end of consumer processing, total time: {} mins",
        (now.elapsed().as_secs_f64() / 60.0)
    );
}

async fn handle_messasge_event(delivery: Delivery, channel: Channel, http_client: reqwest::Client) {
    let string;
    match String::from_utf8(delivery.data) {
        Ok(result) => {
            string = result;
        }
        Err(reason) => {
            println!("could not deserialize message: {}", reason);
            return;
        }
    }
    let message_event =
        from_str::<MessageEvent>(&string).expect("could not deserialize message event");
    handle_message_event_bl(&message_event, &http_client).await;
    println!("message event: {:?}", message_event);
    channel
        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
        .await
        .expect("error while acknowledging the message")
}
