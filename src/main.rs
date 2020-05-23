use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;

use std::fs::OpenOptions;
use std::io::Read;

const TOKEN: &'static str = env!("TOKEN");
const GID:   &'static str = env!("GROUP_ID");

const SEND:   &str = "https://api.vk.com/method/messages.send";
const GET_LP: &str = "https://api.vk.com/method/groups.getLongPollServer";

#[derive(Deserialize)]              //                   LongPollResponse
struct Attachment{                  //                  /                \
    #[serde(rename="type")]         //          ts: event number       updates:[Update]
    type_: String                   //                                            |
}                                   //                                        Message
#[derive(Deserialize)]              //                                       /       \
struct Message{                     //                  peer_id: where to send     attachments:[Attachment]
    peer_id: i32,                   //                                                              |
    attachments: Vec<Attachment>    //                                                             type
}
#[derive(Deserialize)]
struct Update{
    object: Message
}
#[derive(Deserialize)]
struct LongPollResponse{
    ts: String,
    updates: Vec<Update>
}
#[derive(Deserialize)]
struct LongPoll{
    key: String,
    server: String,
    ts: String
}
#[derive(Deserialize)]
struct LongPollError{
    failed: u8,
    ts: Option<String>
}
#[derive(Deserialize)]
struct ResponseVK{
    response: LongPoll
}
fn main() {
    let client = reqwest::blocking::Client::new();
    let (mut key, server, mut  ts) = get_longpoll();
    let mut rng = rand::thread_rng();

    let phrases: Vec<String> = {
        let mut file = OpenOptions::new()
            .read(true)
            .open("phrases.txt")
            .unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        let mut texts: Vec<String> = Vec::new();
        for i in text.split('\n'){
            texts.push(String::from(i))
        }
        texts
        
    };
    loop {
        let mut updates_request = 
        {
            let mut params = HashMap::new();
                params.insert("act","a_check");
                params.insert("key",&key);
                params.insert("ts", &ts);
                params.insert("wait", "25");

            match client.post(&server)
                        .form(&params)
                        .send(){
                Ok(some) => some,
                Err(e) => {
                    eprintln!("Error while getting updates: {}",e);
                    continue
                }
            }
        };
        let longpoll_r: 
            Result<LongPollResponse, _> = serde_json::from_reader(&mut updates_request);
        let updates;
        match longpoll_r {
            Ok(u) => {
                updates=u.updates;
                ts=u.ts;
            }

            Err(_) => {
                let longpoll_err: 
                    LongPollError = serde_json::from_reader(&mut updates_request).unwrap();

                    if longpoll_err.failed == 1 {
                         ts=longpoll_err.ts.unwrap(); 
                         continue
                    }
                    let p=get_longpoll();
                    if longpoll_err.failed == 2 {
                        key=p.0
                    }
                    else if longpoll_err.failed == 3 {
                        key=p.0;
                        ts=p.2;
                    }
                eprintln!("got LongPoll error. Code: {}", longpoll_err.failed);
                continue;
            }
        }
        for u in updates {
            let message = u.object;
            if !message.attachments.is_empty()
                && message.attachments[0].type_ == "audio_message"{
                    {
                        let random_id = rng.gen::<i64>().to_string();
                        let text = phrases.choose(&mut rng).unwrap();
                        let peer_id = message.peer_id.to_string();
                        println!("sending \"{}\" to {}", text, peer_id);

                        let mut params = HashMap::new();
                        params.insert("access_token", TOKEN);
                        params.insert("random_id",&random_id);
                        params.insert("message", &text);
                        params.insert("peer_id",&peer_id);
                        params.insert("v", "5.95");
                        match client.post(SEND)
                                    .form(&params)
                                    .send(){
                                    Ok(_) => {},
                                    Err(e) => eprintln!("Sending error: {}", e)
                                    }
                        
                        
                    
                }
            }
        }
    }

}
fn get_longpoll() -> (String, String, String){
    let client = reqwest::blocking::Client::new();
    
    let mut params = HashMap::new();
    params.insert("group_id",GID);
    params.insert("access_token", TOKEN);
    params.insert("v", "5.95");
    let response = client.post(GET_LP)
        .form(&params)
        .send()
        .unwrap();
    let params:ResponseVK = serde_json::from_str(&response.text().unwrap()).unwrap();
    let lp = params.response;
    (lp.key, lp.server, lp.ts)
}
