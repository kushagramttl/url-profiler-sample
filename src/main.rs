extern crate clap;
use clap::{App, Arg};

use std::io::prelude::*;
use std::net::TcpStream;
use std::str::from_utf8;
use std::time::Instant;

fn main() {
    // By using the clap crate, it is easier to specify the arguments that can be taken by the program.
    let matches = App::new("Url Profiling App")
        .about("The program helps the user profile a particular url")
        .version("0.0.0")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .help("The url on which profile has to be executed")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .help("The profile count for which the url has be called")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    // The domain and route ar splitted in the url to get the host and route for request
    // The host/domain will be used to connect on port 80 for connection.
    let domain_route = url_domain_route_split(matches.value_of("url").unwrap());
    let profile_count: u32 = matches.value_of("profile").unwrap().parse::<u32>().unwrap();
    let mut response_durations: Vec<f32> = Vec::new();
    let mut max_byte_size = std::usize::MIN;
    let mut min_byte_size = std::usize::MAX;
    let mut success_count = 0u32;
    for _ in 0..profile_count {
        // Conection is made with the domain
        match TcpStream::connect(format!("{0}{1}", domain_route.0, ":80")) {
            Ok(mut stream) => {
                let start = Instant::now();
                let _ = stream.write(format!("GET {0} HTTP/1.0\r\n", domain_route.1).as_bytes());
                let _ = stream.write(format!("Host: {0}\r\n\r\n", domain_route.0).as_bytes());
                let mut line = Vec::new();
                let result = stream.read_to_end(&mut line);
                let duration = start.elapsed().as_secs_f32();
                response_durations.push(duration);

                match result {
                    Ok(n) => {
                        let text = from_utf8(&line).unwrap();
                        let res_code = get_response_code(text);
                        if res_code == 200 {
                            success_count += 1;
                            let splitted: Vec<&str> = text.split("\r\n\r\n").collect();
                            println!("{0}", splitted.last().unwrap());
                        } else {
                            println!("Failure error code: {0}", res_code);
                        }
                        max_byte_size = max_byte_size.max(n);
                        min_byte_size = min_byte_size.min(n);
                    }
                    Err(e) => {
                        println!("Error Occured: {0}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error Occured while connecting: {0}", e);
            }
        }
    }

    // calculate the necessties, if not print the default values
    let mut res_min: f32 = std::f32::MAX;
    let mut res_max: f32 = std::f32::MIN;
    let mut total_duration = 0f32;
    let mut response_median = 0f32;
    let mut avarage_duration = 0f32;
    if response_durations.len() > 0 {
        response_durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for dur in response_durations.iter() {
            total_duration += *dur;
            res_min = res_min.min(*dur);
            res_max = res_max.max(*dur);
        }

        response_median = response_durations[response_durations.len() / 2];
        avarage_duration = total_duration / response_durations.len() as f32;
    }

    println!("The number of requests: {0}", profile_count);
    println!("Average duration: {0}", avarage_duration);
    println!("Median response duration: {0}", response_median);
    println!("Minimum response time: {0}", res_min);
    println!("Maximum response time: {0}", res_max);
    println!("Maximum response size: {0}", max_byte_size);
    println!("Minimum response size: {0}", min_byte_size);
    println!(
        "Percentage of requests that succeeded: {0}%",
        (success_count as f32 / profile_count as f32) * 100f32
    );
}

/**
 * Returns the response code from the response recieved from host.
 */
fn get_response_code(response: &str) -> u32 {
    let splitted_response: Vec<&str> = response.split("\r\n").collect();
    let first = splitted_response.first().unwrap();
    let splitted_line: Vec<&str> = first.split(" ").collect();
    let value = splitted_line.get(1).unwrap();
    return value.parse::<u32>().unwrap();
}

/**
 * Returns the domain and route of the url for request
 */
fn url_domain_route_split(url: &str) -> (&str, &str) {
    let mut domain = "";
    let mut route = "";
    if url.contains("//") {
        let split1: Vec<&str> = url.split("//").collect();
        let split2: Vec<&str> = split1.get(1).unwrap().split("/").collect();
        domain = split2.get(0).unwrap();
        let split3: Vec<&str> = url.split(domain).collect();
        route = split3.get(1).unwrap();
    } else {
        let split1: Vec<&str> = url.split("/").collect();
        let split2: Vec<&str> = split1.get(1).unwrap().split("/").collect();
        domain = split2.get(0).unwrap();
        let split3: Vec<&str> = url.split(domain).collect();
        route = split3.get(1).unwrap();
    }
    return (domain, route);
}
