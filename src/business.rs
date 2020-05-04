use crate::dtos::UserDto;
use chat_common_types::events::MessageEvent;
use futures::future::try_join_all;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Response,
};

pub async fn handle_message_event(
    event: &MessageEvent,
    http_client: &reqwest::Client,
) -> Option<()> {
    let user_end_point: String = format!("http://localhost/api/users/{}", event.user_id);
    let client_queues_endpoint = format!(
        "http://localhost/api/rooms/{}/members/queues",
        event.room_id
    );
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kumarmo2_admin"));
    let user_api = http_client.get(&user_end_point).send();
    let client_queues_api = http_client
        .get(&client_queues_endpoint)
        .headers(headers)
        .send();

    let results = try_join_all(vec![user_api, client_queues_api]).await;
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

    // TODO: assemble the data structure and send to all the client queues.
    for (index, response) in responses.into_iter().enumerate() {
        if index == 0 {
            let user = response
                .json::<UserDto>()
                .await
                .expect("could not deserialize user");
            println!("user: {:?}", user);
        } else if index == 1 {
            let member_ids = response
                .json::<Vec<chat_common_types::dtos::Queue>>()
                .await
                .expect("could not deserialize client queue");
            println!("member_ids: {:?}", member_ids);
        }
    }

    Some(())
}
