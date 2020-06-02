// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt

use serde::{Deserialize, Serialize};
use std::time::Instant as time;
use std::io::{Read};
use tiny_http::{StatusCode};


#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub previous: String,
    pub next: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub completions: Vec<String>,
    pub predictions: Vec<String>,
}

/// Start a server that answers JSON suggestion requests with JSON suggestions.
pub fn run(compute_suggestions: impl Fn(Request) -> Response) {
    let compute_answer = move |request: &mut dyn Read| -> std::io::Result<String> {
        let request: Request = serde_json::from_reader(request)?;

        println!("received a prediction request: {:?}", request);
        let start_time = time::now();

        let answer = compute_suggestions(request);

        let duration = (time::now() - start_time).as_secs_f32();
        println!("computed answer in {}s: {:?}", duration, answer);

        Ok(serde_json::to_string(&answer)?)
    };

    let server = tiny_http::Server::http("localhost:3000").unwrap();
    println!("starting server.");

    loop {
        let result = server.recv().map(|mut request| {
            match compute_answer(request.as_reader()) {
                Ok(answer) => request.respond(tiny_http::Response::from_data(answer.as_bytes().to_vec())),
                Err(error) => {
                    eprintln!("Error: {:?}", error);
                    request.respond(tiny_http::Response::empty(StatusCode(500)))
                },
            }
        });

        if let Err(error) = result {
            eprintln!("Error: {:?}", error);
        }
    }
}
