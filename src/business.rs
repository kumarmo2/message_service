use crate::dtos::UserDto;
use chat_common_types::{dtos::Message, events::MessageEvent};
use futures::future::try_join_all;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Response,
};
use serde::{Deserialize, Serialize};

use manager::RabbitMqManager;

//TODO: need to think if we can split the message service into
// message service and user_real_time service. message service will
// only determine what to send and to whom all to send, user_real_time
// service will have all the info about the clients. it will receive the
// data and to which user to send. it will send the data to the user on all devices.

//TODO: need major refactoring.
pub async fn handle_message_event(
    event: &MessageEvent,
    http_client: &reqwest::Client,
    rabbit: RabbitMqManager,
) -> Option<()> {
    let user_end_point: String = format!("http://localhost/api/users/{}", event.user_id);
    let client_queues_endpoint = format!(
        "http://localhost/api/rooms/{}/members/queues",
        event.room_id
    );
    let messge_endpoint = format!("http://localhost/api/messages/{}", event.id);
    // TODO: need to initialize this header man just once.
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kumarmo2_admin"));
    let cloned_headers = headers.clone();
    let user_api = http_client.get(&user_end_point).send();
    let client_queues_api = http_client
        .get(&client_queues_endpoint)
        .headers(headers)
        .send();
    let message_api = http_client
        .get(&messge_endpoint)
        .headers(cloned_headers)
        .send();

    let results = try_join_all(vec![user_api, client_queues_api, message_api]).await;
    let responses: Vec<Response>;
    match results {
        Ok(r) => {
            responses = r;
        }
        Err(reason) => {
            println!("fetching user failed, reason: {}", reason);
            return None;
        }
    }

    let mut client_queues: Vec<chat_common_types::dtos::Queue> = Vec::new();
    // Refactor the code so that I don't have to use this default initialization. its totally wasteful.
    let mut message: Message = Message::default();
    let mut is_message_available = false;

    // // TODO: assemble the data structure and send to all the client queues.
    for (index, response) in responses.into_iter().enumerate() {
        if index == 0 {
            let user = response
                .json::<UserDto>()
                .await
                .expect("could not deserialize user");
            println!("user: {:?}", user);
        } else if index == 1 {
            client_queues = response
                .json::<Vec<chat_common_types::dtos::Queue>>()
                .await
                .expect("could not deserialize client queue");
            println!("member_ids: {:?}", client_queues);
        } else if index == 2 {
            is_message_available = true;
            message = response
                .json::<Message>()
                .await
                .expect("could not deserialize message");
        }
    }
    //TODO: need to make this logic of sending messages very resilient and bug free.
    if is_message_available == true {
        let mut tasks = Vec::new();
        for client_queue in client_queues.iter() {
            let task = rabbit.publish_message_to_queue_async(&client_queue.queue_name, &message);
            tasks.push(task);
        }
        try_join_all(tasks)
            .await
            .expect("sending message to someone failed");
    }

    Some(())
}

#[derive(Serialize, Deserialize)]
struct DummyWrapper {
    content: String,
}
