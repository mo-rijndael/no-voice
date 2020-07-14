use serde::Deserialize;
use rand::Rng;
use rand::seq::SliceRandom;

use std::fs::OpenOptions;
use std::io::Read;
use std::collections::VecDeque;

const TOKEN: &str = env!("TOKEN");
const GID:   &str = env!("GROUP_ID");

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
struct LongPoll{
    key: String,
    server: String,
    ts: String,
    #[serde(skip)]
    cache: VecDeque<Message>,
}
impl LongPoll {
    fn new() -> Self {
        let params = [
            ("group_id",GID), 
            ("access_token", TOKEN),
            ("v", "5.95")
        ];
        let response = ureq::post(GET_LP)
            .send_form(&params)
            .into_string()
            .unwrap();
        let params:ResponseVK = serde_json::from_str(&response).unwrap();
        params.response
    }
    fn get_events(&mut self) {
        let updates_request = 
        {
            let params = [
                ("act","a_check"),
                ("key",&self.key),
                ("ts", &self.ts),
                ("wait", "25")
            ];
            match ureq::post(&self.server)
                        .send_form(&params)
                        .into_string(){
                Ok(some) => some,
                Err(e) => {
                    eprintln!("Error while getting updates: {}",e);
                    return
                }
            }
        };
        let longpoll_r: LongPollResponse = serde_json::from_str(&updates_request).unwrap();
        match longpoll_r {
            LongPollResponse::Normal(u) => {
                self.ts=u.ts;
                self.cache.extend(u.updates.into_iter().map(|x|x.object))
            }

            LongPollResponse::Failed(e) => {
                eprintln!("got LongPoll error. Code: {}", e.failed);
                let new_longpoll = Self::new();
                match e.failed {
                    1 => {self.ts = e.ts.unwrap()}
                    2 => {self.key = new_longpoll.key}
                    3 => {self.key = new_longpoll.key;
                          self.ts = new_longpoll.ts}
                    _ => {}
                    
                }
                return
            }
        };
    }
}
impl std::iter::Iterator for LongPoll {
    type Item = Message;
    fn next(&mut self) -> Option<Self::Item> {
        while self.cache.is_empty() {
            self.get_events()
        }
        self.cache.pop_front()
    }
}
#[derive(Deserialize)]
struct LongPollOk{
    ts: String,
    updates: Vec<Update>
}
#[derive(Deserialize)]
struct LongPollError{
    failed: u8,
    ts: Option<String>
}
#[derive(Deserialize)]
#[serde(untagged)]
enum LongPollResponse {
    Normal(LongPollOk),
    Failed(LongPollError),
}
#[derive(Deserialize)]
struct ResponseVK{
    response: LongPoll
}
fn main() {
    let long_poll = LongPoll::new();
    let mut rng = rand::thread_rng();

    let phrases: Vec<String> = {
        let mut file = OpenOptions::new()
            .read(true)
            .open("phrases.txt")
            .unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        text.split(' ')
            .map(String::from)
            .collect()
    };
    for message in long_poll {
        if !message.attachments.is_empty()
        && message.attachments[0].type_ == "audio_message"{
                {
                    let random_id = rng.gen::<i64>().to_string();
                    let text = phrases.choose(&mut rng).unwrap();
                    let peer_id = message.peer_id.to_string();
                    println!("sending \"{}\" to {}", text, peer_id);
                    let params = [
                        ("access_token", TOKEN),
                        ("random_id",&random_id),
                        ("message", &text),
                        ("peer_id",&peer_id),
                        ("v", "5.95")
                    ];
                    if ureq::post(SEND)
                                .send_form(&params)
                                .error(){
                                    eprintln!("Sending error");
                    }
                    
                    
                
            }
        }
    }

}
