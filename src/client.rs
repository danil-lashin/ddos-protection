mod miner;

pub mod pb {
    tonic::include_proto!("grpc.api");
}

use std::env;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::miner::check_solution;
use pb::{api_client::ApiClient, ChallengeRequest, QuoteRequest};

async fn get_token(client: &mut ApiClient<Channel>) -> Result<String, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(128);
    let out_stream = ReceiverStream::new(rx);

    println!("Sending challenge request");
    let response = client.challenge(out_stream).await.unwrap();
    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();

        if received.token != "" {
            println!("Received token: {}", received.token);
            return Ok(received.token);
        }

        println!("Received challenge: {:?}", received);

        let mut i = 0;
        loop {
            if check_solution(i, received.difficulty, received.challenge) {
                println!("Found solution: {}", i);
                tx.send(ChallengeRequest { solution: i }).await.unwrap();
                break;
            }

            i += 1;
        }
    }

    return Err(Box::try_from("No token received").unwrap());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ApiClient::connect(env::var("API_HOST").unwrap()).await?;

    let token = get_token(&mut client).await?;
    let quote = client
        .quote(QuoteRequest {
            token: token.to_string(),
        })
        .await?
        .into_inner()
        .quote;

    println!("{}", quote);

    Ok(())
}
