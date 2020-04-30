Bot, that becomes triggered by voice messages
=====================================
Voice messages are evil. It costs too much time to listen it. People, who use voice message are selfish ~~idiots~~.
So, this bot won't help you recognize voice to text. But it can rudely convey, that voices are inexcusable directly to chat members.

I need it for my chat!!!!
=========================
This bot works in russian language in vk.com. Add [this bot](https://vk.com/fuck_voices) to your chat. 

I want make my bot
==================
1. install [rust toolchain](https://rust-lang.org) 
2. get access token with permissions `messages` and `manage` in group settings
3. set environment variables `TOKEN` and `GROUP_ID` (group id without -)
4. `cargo build --release`
5. put phrases in `phrases.txt` next to executable file. One phrase per line.
