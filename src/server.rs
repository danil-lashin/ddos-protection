pub mod pb {
    tonic::include_proto!("grpc.api");
}

mod miner;
mod quotes;
mod tokens;

use futures::{Stream};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{net::ToSocketAddrs, pin::Pin};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Code, Request, Response, Status, Streaming};

use crate::pb::{QuoteRequest, QuoteResponse};
use crate::quotes::fetch_quote;
use crate::tokens::{check_token, generate_token};
use pb::{ChallengeRequest, ChallengeResponse};

const SLOTS: usize = 10;
const DIFFICULTY: u64 = u64::MAX / 1000; // todo: make this dynamic?

fn calculate_target_work_count(requests: u64) -> u64 {
    requests/100 + 1000 // todo: calculate this based on the number of requests
}

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<ChallengeResponse, Status>> + Send>>;

#[derive(Debug)]
pub struct ApiServer {
    requests_counter: Arc<Mutex<[u64; SLOTS]>>,
}

#[tonic::async_trait]
impl pb::api_server::Api for ApiServer {
    type ChallengeStream = ResponseStream;

    async fn challenge(
        &self,
        req: Request<Streaming<ChallengeRequest>>,
    ) -> EchoResult<Self::ChallengeStream> {
        let mut in_stream = req.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let counter = self.requests_counter.clone();

        tokio::spawn(async move {
            let mut total_requests = 0;
            {
                let counter = counter.lock().unwrap();
                for num in counter.iter() {
                    total_requests += num;
                }
            }

            let mut target_iterations = calculate_target_work_count(total_requests);
            let mut current_iteration = 0u64;
            let mut challenge = random_challenge();

            // send initial challenge
            tx.send(Ok(ChallengeResponse {
                challenge,
                left: target_iterations - current_iteration,
                difficulty: DIFFICULTY,
                token: "".to_string(),
            }))
            .await
            .expect("working rx");

            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => {
                        if !miner::check_solution(v.solution, DIFFICULTY, challenge) {
                            tx.send(Err(Status::new(Code::InvalidArgument, "Invalid solution")))
                                .await
                                .expect("working rx");
                            break;
                        }

                        current_iteration += 1;
                        challenge = random_challenge();

                        let mut total_requests = 0;
                        {
                            let mut counter = counter.lock().unwrap();
                            let current_sec = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            counter[(current_sec % SLOTS as u64) as usize] += 1;
                            counter[((current_sec + 1) % SLOTS as u64) as usize] = 0;
                            for num in counter.iter() {
                                total_requests += num;
                            }
                        }

                        target_iterations = calculate_target_work_count(total_requests);

                        if current_iteration >= target_iterations {
                            tx.send(Ok(ChallengeResponse {
                                challenge,
                                left: 0,
                                difficulty: DIFFICULTY,
                                token: generate_token(),
                            }))
                            .await
                            .expect("working rx");
                            break;
                        }

                        tx.send(Ok(ChallengeResponse {
                            challenge,
                            left: target_iterations - current_iteration,
                            difficulty: DIFFICULTY,
                            token: "".into(),
                        }))
                        .await
                        .expect("working rx")
                    }
                    Err(err) => match tx.send(Err(err)).await {
                        Ok(_) => (),
                        Err(_err) => break,
                    },
                }
            }
        });

        let out_stream = ReceiverStream::new(rx);
        // todo: close connection if no requests for 10 seconds

        Ok(Response::new(Box::pin(out_stream) as Self::ChallengeStream))
    }

    async fn quote(
        &self,
        request: Request<QuoteRequest>,
    ) -> Result<Response<QuoteResponse>, Status> {
        if check_token(request.into_inner().token) {
            // fetch random quote
            let quote = fetch_quote().await;
            Ok(Response::new(QuoteResponse { quote }))
        } else {
            Err(Status::new(Code::InvalidArgument, "Invalid token"))
        }
    }
}

fn random_challenge() -> u64 {
    Rng::gen(&mut rand::thread_rng())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = pb::api_server::ApiServer::new(ApiServer {
        requests_counter: Arc::new(Mutex::new([0; SLOTS])),
    });

    Server::builder()
        .add_service(server)
        .serve("0.0.0.0:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}
