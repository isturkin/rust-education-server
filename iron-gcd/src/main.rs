extern crate iron;
extern crate reqwest;
extern crate router;
#[macro_use] extern crate mime;

use std::str::FromStr;
use iron::prelude::*;
use iron::status;
use router::Router;
use urlencoded::UrlEncodedBody;

use serde::Deserialize;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;

use tokio::runtime::Runtime;


fn main() {
    let mut router = Router::new();

    router.get("/", get_form, "root");
    router.get("/test", async_external_handler, "test");
    router.post("/gcd", post_gcd, "gcd");

    println!("Serving on http://localhost:3000...");
    Iron::new(router).http("localhost:3000").unwrap();
}

fn get_form(_request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(r#"
        <title>GCD Calculator</title>
        <form action="/gcd" method="post">
            <input type="text" name="n"/>
            <input type="text" name="n"/>
            <button type="submit">Compute GCD</button>
        </form>
        "#);

    Ok(response)
}

#[derive(Deserialize, Debug)]
struct ExternalUser {
    id: i32,
    name: String,
    email: String
}


fn async_external_handler(_req: &mut Request) -> IronResult<Response> {
    // Создаем async runtime внутри handler
    let rt = Runtime::new().unwrap();

    let client = reqwest::Client::new();
    
    let response = rt.block_on(async {
        let url = "http://127.0.0.1:8080/users/123";
        client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Reqwest error: {}", e))?
            .json::<ExternalUser>()
            .await
            .map_err(|e| format!("JSON error: {}", e))
    });
    
    match response {
        Ok(user) => Ok(Response::with((
            status::Ok,
            format!("External user: id={}, name={}", user.id, user.name)
        ))),
        Err(e) => Ok(Response::with((
            status::InternalServerError,
            format!("Async error: {}", e)
        )))
    }
}

fn post_gcd(request: &mut Request) -> IronResult<Response> {
    let mut response = Response::new();

    // пример pattern matching'а
    let form_data = match request.get_ref::<UrlEncodedBody>() { // указываем, какую часть запроса получить с помощью параметрического типа UrlEncodedBody
        Err(e) => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("Error parsing form data: {:?}\n", e));
            return Ok(response);
        }
        Ok(map) => map
    };

    let unparsed_numbers = match form_data.get("n") {
        None => {
            response.set_mut(status::BadRequest);
            response.set_mut(format!("form data has no 'n' parameter!\n"));
            return Ok(response);
        }
        Some(nums) => nums
    };

    let mut numbers = Vec::new();
    for unparsed in unparsed_numbers {
        match u64::from_str(&unparsed) {
            Err(_) => {
                response.set_mut(status::BadRequest);
                response.set_mut(
                    format!("Value 'n' is not a number!")
                );
                return Ok(response);
            }
            Ok(n) => { numbers.push(n); }
        }
    }

    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    response.set_mut(status::Ok);
    response.set_mut(mime!(Text/Html; Charset=Utf8));
    response.set_mut(
        format!("The greatest common divisor of the numbers {:?} is <b>{}</b>\n", numbers, d)
    );
    Ok(response)
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m!= 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}
