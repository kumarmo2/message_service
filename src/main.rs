use chat_common_types::events::MessageEvent;
use lapin::{message::Delivery, options::*, types::FieldTable, Channel, Consumer};
use manager::{ChannelManager, PooledConnection, RabbitMqManager};
use num_cpus;
use serde_json::from_str;
use smol::{self, block_on, Task};
use std::time::Instant;

fn main() {
    println!("hello world");
    // TODO: retrieve from the config based on environment.
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    // TODO: update the RabbitMqManager to accept the max number of connections and channels.
    let rabbit = RabbitMqManager::new(addr);
    let cpus = num_cpus::get().max(1);
    for _ in 0..cpus {
        std::thread::spawn(move || smol::run(futures::future::pending::<()>()));
    }

    block_on(start_consumer(rabbit, "messages"));
}

async fn start_consumer(rabbit: RabbitMqManager, queue_name: &'static str) {
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
        process_messages(consumer, channel).await;
    }
}

async fn process_messages(consumer: Consumer, channel: PooledConnection<ChannelManager>) {
    println!("start of consumer processing");
    let now = Instant::now();
    for delivery in consumer {
        println!(
            "message received, thread_id: {:?}",
            std::thread::current().id()
        );
        let d: Delivery;
        if let Ok(delivery) = delivery {
            Task::spawn(handle_messasge_event(delivery, channel.clone())).detach()
        }
    }
    println!(
        "end of consumer processing, total time: {} mins",
        (now.elapsed().as_secs_f64() / 60.0)
    );
}

async fn handle_messasge_event(delivery: Delivery, channel: Channel) {
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
    let message_event = from_str::<MessageEvent>(&string);
    println!("message event: {:?}", message_event);
    channel
        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
        .await
        .expect("error while acknowledging the message")
}
