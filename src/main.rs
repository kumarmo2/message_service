use chat_common_types::events::MessageEvent;
use lapin::{message::Delivery, options::*, types::FieldTable, Consumer};
use manager::{ChannelManager, PooledConnection, RabbitMqManager};
use num_cpus;
use serde_json::from_str;
use smol::{self, block_on, Task};

fn main() {
    println!("hello world");
    // TODO: retrieve from the config based on environment.
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let rabbit = RabbitMqManager::new(addr);
    // let rabbit_sub_cloned = rabbit.clone();
    let cpus = num_cpus::get().max(1);
    for _ in 0..cpus {
        std::thread::spawn(move || smol::run(futures::future::pending::<()>()));
    }

    block_on(start_consumer(rabbit, "messages"));
}

async fn start_consumer(rabbit: RabbitMqManager, queue_name: &'static str) {
    loop {
        let channel: PooledConnection<ChannelManager> = rabbit.get_channel_pool().get().unwrap();
        println!("got channel");
        // let channel = rabbit.get_channel_pool().get().unwrap();
        let consumer: Consumer = channel
            .basic_consume(
                queue_name,
                "message_service",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
        println!("message received");
        Task::spawn(process_messages(consumer, channel)).detach();
    }
}

async fn process_messages(consumer: Consumer, channel: PooledConnection<ChannelManager>) {
    for delivery in consumer {
        println!(
            "message received, thread_id: {:?}",
            std::thread::current().id()
        );
        let d: Delivery;
        if let Ok(delivery) = delivery {
            d = delivery;
            let string;
            match String::from_utf8(d.data) {
                Ok(result) => {
                    string = result;
                }
                Err(reason) => {
                    println!("could not deserialize message: {}", reason);
                    break;
                }
            }
            let message_event = from_str::<MessageEvent>(&string);
            println!("message event: {:?}", message_event);
            channel
                .basic_ack(d.delivery_tag, BasicAckOptions::default())
                .await
                .expect("error while acknowledging the message")
        }
    }
}
