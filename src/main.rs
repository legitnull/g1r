use std::io::prelude::*;
use std::net::TcpStream;
use std::io::Write;
use openssl::ssl::{SslMethod, SslConnector};
use toml::Value;
use serde::Deserialize;
use colored::*;


mod modules {
    pub trait Command {
        fn handle(&self, message: &str) -> Vec<String>;
    }
    pub mod ping;
    pub mod kill;
    pub mod ai;
    pub mod invade;
    pub mod test;
    //pub mod ai_invade;
}

use modules::ai::Ai; // FIX THIS BS
use modules::ping::PingCommand;
use modules::invade::InvadeCommand;
//use modules::ai_invade::AiInvadeCommand;
use modules::kill::KillCommand; // ...
use crate::modules::Command;

#[derive(Deserialize)]
struct Config {
    server: String,
    port: u16,
    nick: String,
    password: String,
    channels: Vec<String>,
    admin_users: Vec<String>,
    ignore_users: Vec<String>,
    
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // LOAD CONFIG
    let config_str = std::fs::read_to_string("config.toml").unwrap();
    let config_value = config_str.parse::<Value>().unwrap();
    let config: Config = config_value.try_into().unwrap();
    // GIVE THE SERVER A SLOPPPY SPAM OF RETARDEDNESS
    let stream = TcpStream::connect(format!("{}:{}", config.server, config.port)).unwrap(); 
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    // DONT DO DRUGS YOU WILL END UP LIKE ME
    let mut ssl_stream = connector.connect(&config.server, stream).unwrap();
    let nick_command = format!("NICK {}_\r\n", config.nick); //setup passwords
    let user_command = format!("USER {} 0 * :{}\r\n", config.nick, config.nick);
    ssl_stream.write_all(nick_command.as_bytes()).unwrap();
    ssl_stream.write_all(user_command.as_bytes()).unwrap();
    let identify_command = format!("PRIVMSG NickServ :IDENTIFY {} {}\r\n", config.nick, config.password);
    ssl_stream.write(identify_command.as_bytes()).unwrap();
    let channels = config.channels.join(",");
    let join_command = format!("JOIN {}\r\n", channels);
    
    let admin_users = config.admin_users; // ADMINS
    let ignored_users = config.ignore_users; // IGNORED
// ... 
    ssl_stream.write_all(join_command.as_bytes()).unwrap();

    let mut buf = [0; 512];
    loop {
        match ssl_stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let received = String::from_utf8_lossy(&buf[0..n]);
                let message = received.trim();

                //debug chat 
                println!("{} {}","[%] DEBUG:".bold().green(), received.purple());


                // RESPOND TO PINGS
                if message.starts_with("PING") {
                    println!("{} {}","[%] PONG:".bold().green(), config.nick.blue()); // DEBUG
                    ssl_stream.write_all("PONG ircd.chat\r\n".as_bytes()).unwrap();
                    continue; // skip processing the PING message further
                }

                // MODULES
                let ping_command = PingCommand;
                let kill_command = KillCommand;
                let invade_command = InvadeCommand;
                //let ai_invade_command = AiInvadeCommand;

                //let test_command = TestCommand;
                let ai = Ai;

                // ADMIN MODULES
                if message.starts_with(":") && message.contains(" :%") {
                    let parts: Vec<&str> = message.splitn(2, ' ').collect(); // Check if user is admin_user
                    let username = parts[0].trim_start_matches(':').split("!").next().unwrap();
                    if !admin_users.contains(&username.to_string()) {
                        println!("{} {}","[!] UNAUTHORIZED:".bold().clear().on_red(), username.red().bold());
                        continue; // ...
                    }
                    if message.contains(":%ping") {
                        for response in ping_command.handle(message) {
                            ssl_stream.write_all(response.as_bytes()).unwrap();
                        }
                    } else if message.contains(":%kill") {
                        for response in kill_command.handle(message) {
                            ssl_stream.write_all(response.as_bytes()).unwrap();
                        }
                    } else if message.contains(":%invade") {
                        for response in invade_command.handle(message) {
                            ssl_stream.write_all(response.as_bytes()).unwrap();
                        }
                    } //else if message.contains(":%aiinvade") {
                      //  for response in ai_invade_command.handle(message) {
                      //      ssl_stream.write_all(response.as_bytes()).unwrap();
                      //  }
                    //} 
                }

                // Check if the message is user and respond via ai
                else if message.starts_with(":") && message.contains("PRIVMSG ") && message.contains(&config.nick) { //modify for on mention 
                    let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap();
                    if !channels.contains(&channel) {
                        continue;
                    }
                    // extract the username from the first part and check if ignored
                    let parts: Vec<&str> = message.splitn(2, ' ').collect(); // split the message into two parts at the first space
                    let username = parts[0].trim_start_matches(':').split("!").next().unwrap();
                    if ignored_users.contains(&username.to_string()) {
                        println!("[!] IGNORED: {}", username.red()); 
                        continue;
                    }
                    for response in ai.handle(message, ) {
                        ssl_stream.write_all(response.as_bytes()).unwrap();
                    }

                }

            },
            Err(e) => {
                println!("[!] ERROR: {}", e);
                break;
            },
        }
    }
}
