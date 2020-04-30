use futures::future::join_all;
use lapin::{options::*, types::FieldTable};
use manager::RabbitMqManager;
use smol::{self, Task};

fn main() {
    println!("hello world");
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let rabbit = RabbitMqManager::new(addr);
    let rabbit_sub_cloned = rabbit.clone();

    smol::run(async move {
        let consumer_task = start_consumer(rabbit_sub_cloned, "hello");
        join_all(vec![consumer_task]).await;
    });
}

fn start_consumer(rabbit: RabbitMqManager, queue_name: &'static str) -> Task<()> {
    return Task::spawn(async move {
        loop {
            let channel = rabbit.get_channel_pool().get().unwrap();
            let consumer = channel
                .basic_consume(
                    queue_name,
                    "message_service",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();
            for delivery in consumer {
                println!("message received",);
                if let Ok(delivery) = delivery {
                    channel
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                        .await
                        .expect("error while acknowledging the message")
                }
            }
        }
    });
}

// fn start_publisher(rabbit: RabbitMqManager) -> Task<()> {
//     // let rabbit_pub_cloned = rabbit.clone();
//     return Task::spawn(async move {
//         // let publish_timeout = Duration::from_millis(2000);
//         let mut counter = 0;
//         loop {
//             let result = rabbit
//                 .publish_message_to_queue_async(
//                     "hello",
//                     &DummyPayload {
//                         input: "kumarmo2".to_string(),
//                     },
//                 )
//                 .await;
//             match result {
//                 Ok(is_published) => match is_published {
//                     true => {
//                         println!("message sent: {}", counter);
//                         counter = counter + 1;
//                     }
//                     false => println!("error confirming message publish"),
//                 },
//                 Err(reason) => {
//                     println!("error occured: {:?}", reason);
//                 }
//             }
//             // Timer::after(publish_timeout).await;
//         }
//     });
// }

// #[derive(Serialize)]
// struct DummyPayload {
//     input: String,
// }
