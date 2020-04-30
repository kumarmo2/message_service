use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use manager::RabbitMqManager;
use serde::Serialize;
use smol::block_on;
use smol::Task;
use smol::Timer;
use std::time::Duration;

fn main() {
    println!("hello world");
    //
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let mut rabbit = RabbitMqManager::new(addr);

    // Task::spawn(async move {
    block_on(async move {
        let publish_timeout = Duration::from_millis(200);
        loop {
            let result = rabbit
                .publish_message_to_queue_async(
                    "hello",
                    &DummyPayload {
                        input: "kumarmo2".to_string(),
                    },
                )
                .await;
            match result {
                Ok(is_published) => match is_published {
                    true => println!("messaged was published with confirm"),
                    false => println!("error confirming message publish"),
                },
                Err(reason) => {
                    println!("error occured: {:?}", reason);
                }
            }
            println!("Going to sleep");
            Timer::after(publish_timeout).await;
            println!("woke up");
        }
    });
}

#[derive(Serialize)]
struct DummyPayload {
    input: String,
}

// ------------- using asynchronous apis in synchronous environment example------------

// use futures::future::join_all;
// use std::time::Instant;
// use tokio::fs;
// use tokio::runtime::Runtime;

// fn main() {
//     println!("hello world");
//     let mut runtime = Runtime::new().expect("could not create runtime");
//     let files = vec!["./some.iso", "./movie2.mp4", "./movie3.mp4", "./movie4.mp4"];
//     //read_files_one_by_one(&mut runtime, &files);
//     read_files_in_parallel(&mut runtime, &files);
// }

// fn read_files_one_by_one(runtime: &mut Runtime, paths: &[&str]) {
//     println!("starting reading files one by one");
//     let start = Instant::now();
//     for path in paths {
//         let fut = fs::read(path);
//         let data = runtime.block_on(fut).unwrap();
//         // println!("done reading: {}, len: {}", path, data.len());
//     }
//     println!(
//         "done reading all the files. total time: {}",
//         start.elapsed().as_millis()
//     );
// }

// fn read_files_in_parallel(runtime: &mut Runtime, paths: &[&str]) {
//     // let x = paths.iter().map(|path| fs::read(path)).collect::<Future>();
//     let futures = paths.iter().map(|path| fs::read(path));
//     println!("starting reading files in parallel");
//     let start = Instant::now();
//     let results = runtime.block_on(join_all(futures));
//     println!(
//         "done reading all the files. total time: {}",
//         start.elapsed().as_millis()
//     );
// }
