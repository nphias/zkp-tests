use num::BigUint;
use std::io::{stdin, stdout, Write};
use std::env;

pub mod zkp_auth {
    include!("../zkp_auth.rs");
}

use zkp_auth::auth_client::AuthClient;
use zkp_auth::{AuthenticationAnswerRequest, AuthenticationChallengeRequest, RegisterRequest};

use cp_zkp::{
    parse_group_from_command_line, get_constants, get_random_number, exponentiates_points,
    solve_zk_challenge_s,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let group = parse_group_from_command_line(args);

    let server_addr = "http://0.0.0.0:50051";

    println!(
        "Running client connecting to {} ZKP: {:?}",
        server_addr, group
    );

    let mut client = AuthClient::connect(server_addr).await?;
    println!();
    println!("BEGIN REGISTRATION PROCESS");
    let (p, q, g, h) = get_constants(&group);
    print!("[CLIENT] constants P: {:?}", p); //10009
    print!(" Q: {:?}", q); //5004
    print!(" G: {:?}", g.serialize()); //[3]
    println!(" H: {:?}", h.serialize()); //[11,76]

    'main_loop: loop {
        let x = get_random_number();
        println!("[CLIENT] random x: {:?}", x);

        // (y1, y2) = (g^x, h^x) secret x
        let (y1, y2) = exponentiates_points(&x, &g, &h, &p).unwrap();
        
        print!("[CLIENT] x,G,H,P exponents (modpow) y1: {:?}", y1.serialize());
        println!(" y2: {:?}", y2.serialize());
        println!("Enter your name to register");

        let mut stdin_string = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut stdin_string)
            .expect("Did not enter a correct string");
        let user_name = stdin_string.trim().to_string();

        let server_response = client
            .register(RegisterRequest {
                user: user_name.clone(),
                y1: y1.serialize(),
                y2: y2.serialize(),
            })
            .await;

        match server_response {
            Ok(_) => {
                println!("[SERVER] registration success");
            }
            Err(server_response) => {
                println!(
                    "[SERVER] Error occurred during registration: {:?}",
                    server_response.message()
                );
                break 'main_loop;
            }
        }
        println!();
        
        // (r1, r2) = (g^k, h^k) random k
        println!("BEGIN LOGIN PROCESS");

        let k = get_random_number();
        println!("[CLIENT] random k: {:?}", k);

        let (r1, r2) = exponentiates_points(&k, &g, &h, &p).unwrap();
        print!("[CLIENT] k,G,H,P exponents (modpow) r1: {:?}", r1.serialize());
        println!(" r2: {:?}", r2.serialize());
        println!("Enter to send Login request for {} or 'q' to quit",user_name);

        let mut stdin_string = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut stdin_string)
            .expect("Did not enter a correct string");
        match stdin_string.trim() {
            "q" | "Q" => {
                break 'main_loop;
            }
            _ => {
            }
        }
        
        let server_response = client
            .create_authentication_challenge(AuthenticationChallengeRequest {
                user: user_name,
                r1: r1.serialize(),
                r2: r2.serialize(),
            })
            .await;

        if let Err(registration_response) = &server_response {
            println!(
                "[SERVER] Error occurred during challenge request: {:?}",
                registration_response.message()
            );
            break 'main_loop;
        }

        let response = server_response?.into_inner();
        let auth_id = response.auth_id;

        let c = response.c;
        let c = BigUint::from_bytes_be(&c);
        let s = solve_zk_challenge_s(&x, &k, &c, &q);
        print!("[SERVER] Challenge response, Auth ID: {}", auth_id);
        println!(" Random c: {}", c);
        println!();
        println!("[CLIENT] x,k,c,Q Challenge solved: {:?}",s);
        println!("Enter to send challenge verification: {} or 'q' to quit",s);

        let mut stdin_string = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut stdin_string)
            .expect("Did not enter a correct string");
        match stdin_string.trim() {
            "q" | "Q" => {
                break 'main_loop;
            }
            _ => {
            }
        }

        let server_response = client
            .verify_authentication(AuthenticationAnswerRequest {
                auth_id,
                s: s.to_bytes_be(),
            })
            .await;

        match server_response {
            Ok(auth_response) => {
                println!(
                    "[SERVER] Success, logged in with Session ID: {:?}\n",
                    auth_response.into_inner().session_id
                )
            }
            Err(auth_response) => {
                println!(
                    "[SERVER] Error occurred (server response): {:?}\n",
                    auth_response.message()
                )
            }
        }
        break 'main_loop;
    }
    Ok(())
}
