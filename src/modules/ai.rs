// Check if the message is user and respond via ai
use async_openai::{Client, types::{CreateCompletionRequestArgs}};
use regex::Regex;
use crate::modules::Command;
use toml::Value;
use serde::Deserialize;
use colored::*;

#[derive(Deserialize)]
struct Config {
    nick: String,
    openai: String,
    model: String,
    accents: String,
    personalities: String,
}
pub struct Ai;
   // setup a prompt and respnse log for training other bots
impl Command for Ai {
    fn handle(&self, message: &str) -> Vec<String> {
        let mut responses = Vec::new();
        let config_str = std::fs::read_to_string("config.toml").unwrap();
        let config_value = config_str.parse::<Value>().unwrap();
        let config: Config = config_value.try_into().unwrap(); // respond to name with and without leet VVV
        if message.starts_with(":") && message.contains("PRIVMSG ") && message.contains(&config.nick) { 
            let channel = message.split("PRIVMSG ").nth(1).and_then(|s| s.splitn(2, ' ').next()).unwrap(); // set the response to varible
            let user_message = format!("The following is a chat log:\n{}\nRespond {} as you are chatting as {}: \n\n", 
                message.split(&format!("PRIVMSG {} :", channel.to_string())).nth(1).unwrap(),
                config.accents,
                config.personalities
            );                    
            let parts: Vec<&str> = message.splitn(2, ' ').collect();    
            let username = parts[0].trim_start_matches(':').split("!").next().unwrap();

            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(ai(&user_message, &username, &channel));
            responses.extend(result);
        }

        responses
    }
}
async fn ai(user_message: &str, username: &str, channel: &str) -> Vec<String> {
    let config_str = std::fs::read_to_string("config.toml").unwrap();
    let config_value = config_str.parse::<Value>().unwrap();
    let config: Config = config_value.try_into().unwrap();
    let api_key = config.openai; // set this from config

    let client = Client::new().with_api_key(api_key);
    println!("{} {} {}: {}", "[?]".on_green().bold(), "PROMPT:".green().bold(), username, user_message);
    let chat_request = CreateCompletionRequestArgs::default()
        .prompt(user_message)
        .max_tokens(40_u16)
        .model(config.model)
        .build()
        .unwrap();
    let chat_response = client
        .completions()
        .create(chat_request)
        .await
        .unwrap();
    println!("{} {} {}","[+]".on_green().bold(), "RESPONSE:".green().bold(), chat_response.choices.first().unwrap().text);
    //modify regex for varible username ie G1R g1r GIR gir but as handle nick for bots
    let response_text = &chat_response.choices.first().unwrap().text;
    let regex = Regex::new(r#""|[gG][1iI][rR]:\s"#).unwrap(); // THIS IS FUCKING UP EVERYTHING
    //let nick = &config.nick;
    //let regex_str = format!(
    //    r#""|[{}{}{}]|\b[gG][1iI][rR]:\s*|\b[mM][eE]:?\s"#,
    //    nick.to_lowercase(),
    //    nick.to_uppercase(),
    //    nick.chars().map(|c| match c { /// regex magic nick removal in progress
    //        'a' => '4',
    //        'e' => '3',
    //        'i' => '1',
    //        'o' => '0',
    //        's' => '5',
    //        _ => c,
    //    }).collect::<String>(),
    //);
    //let regex = Regex::new(&regex_str).unwrap();
    let response_text = regex.replace_all(response_text, "").trim().to_string();
    let response_lines = response_text.split("\n").filter(|line| !line.trim().is_empty());
    let mut responses = Vec::new();
    for line in response_lines {
        responses.push(format!("PRIVMSG {} :\x0313{}\x0f: {}\r\n", channel, username, line));
    }
    
    responses
}
