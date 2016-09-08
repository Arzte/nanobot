// ISC License (ISC)
//
// Copyright (c) 2016, Austin Hellyer <hello@austinhellyer.me>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

use chrono::UTC;
use discord::model::{ChannelId, ServerId, UserId, permissions};
use discord::ChannelRef;
use std::collections::{BTreeMap, HashMap};
use std::default::Default;
use ::prelude::*;
use ::ext::youtube_dl::{self, Response as YoutubeDLResponse};

fn get_duration(secs: u64) -> String {
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum SkipVote {
    AlreadyVoted,
    Passed,
    Voted,
    VoterSkipped,
}

#[derive(Clone, Debug)]
pub struct MusicPlaying {
    pub req: MusicRequest,
    pub skip_votes_required: u16,
    pub skip_votes: Vec<UserId>,
    pub started_at: u64,
}

#[derive(Clone, Debug)]
pub struct MusicRequest {
    pub response: YoutubeDLResponse,
    pub requested_in: ChannelId,
    pub requester_name: String,
    pub requester: UserId,
}

impl MusicRequest {
    pub fn format_duration(&self) -> String {
        get_duration(self.response.data.duration)
    }
}

#[derive(Clone, Debug)]
pub struct MusicState {
    pub queue: HashMap<ServerId, Vec<MusicRequest>>,
    pub song_completion: BTreeMap<u64, Vec<ServerId>>,

    /// A list of the playing status of each server. When the thread is checking
    /// the `play_queue`, it should be double-checked here that the server is
    /// still in this queue - regardless of status - as its key is removed on
    /// voice disconnect.
    ///
    /// Alternatively, `queue` can be checked.
    ///
    /// The key should be updated to `None` on a successful thread check if
    /// there is no requests in the `queue`.
    pub status: HashMap<ServerId, Option<MusicPlaying>>,
}

impl Default for MusicState {
    fn default() -> MusicState {
        MusicState {
            song_completion: BTreeMap::new(),
            status: HashMap::new(),
            queue: HashMap::new(),
        }
    }
}

#[allow(or_fun_call)]
pub fn join(context: Context) {
    let text = context.text(0);

    let state = context.state.lock().unwrap();
    let (channel_id, server_id) = if !text.is_empty() {
        let mentions = context.channel_mentions();

        match mentions.get(0) {
            Some(channel) => (channel.id, channel.server_id),
            None => {
                let _msg = req!(context.say("Must mention a channel or be in one"));

                return;
            },
        }
    } else {
        let finding = state.find_voice_user(context.message.author.id);

        match finding {
            Some((Some(server_id), channel_id)) => (channel_id, server_id),
            Some((None, _channel_id)) => {
                let _msg = req!(context.say("Groups not supported"));

                return;
            },
            None => {
                let _msg = req!(context.say("Must mention a channel or be in one"));

                return;
            },
        }
    };
    drop(state);

    let mut state = context.music_state.lock().unwrap();

    // Check if we're already in a voice channel in the server
    if state.status.contains_key(&server_id) {
        let _msg = req!(context.say("Already in a voice channel"));

        return;
    }

    {
        let mut conn = context.conn.lock().unwrap();

        {
            let voice = conn.voice(Some(server_id));
            voice.set_deaf(true);
            voice.connect(channel_id);
        }

        drop(conn);
    }

    state.status.insert(server_id, None);
    let _ = state.queue.entry(server_id).or_insert(vec![]);

    drop(state);

    let _msg = req!(context.say("Ready to play audio"));
}

pub fn leave(context: Context) {
    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    {
        let mut conn = context.conn.lock().unwrap();
        conn.drop_voice(Some(server_id));
    }

    let mut state = context.music_state.lock().unwrap();

    state.status.remove(&server_id);
    state.queue.remove(&server_id);

    drop(state);

    let _msg = req!(context.say("Left the voice channel"));
}

#[allow(or_fun_call)]
pub fn play(context: Context) {
    let server_id = {
        let data_state = context.state.lock().unwrap();
        match data_state.find_channel(&context.message.channel_id) {
            Some(ChannelRef::Public(server, _channel)) => server.id,
            _ => {
                let _msg = req!(context.say("Could not find server"));

                return;
            },
        }
    };

    // Check if a URL is given. If not, then we need only join the voice
    // channel they are in.
    let url = context.text(0);

    let mut state = context.music_state.lock().unwrap();

    // Attempt to join the user's voice channel _if_ - and _only_ if - we
    // are not already in one.
    //
    // If they are not in one, then we can still try to queue their song. If
    // there is no song, then it will simply be time to exit.
    if !state.status.contains_key(&server_id) {
        let data_state = context.state.lock().unwrap();

        match data_state.find_voice_user(context.message.author.id) {
            Some((Some(_server_id), channel_id)) => {
                let mut conn = context.conn.lock().unwrap();
                {
                    let voice = conn.voice(Some(server_id));

                    // Only connect to the user's voice channel if we're not
                    // already in one.
                    if let None = voice.current_channel() {
                        voice.connect(channel_id);
                        voice.set_deaf(true);
                    }
                }

                drop(conn);
            },
            _ => {
                // If no URL was provided, let them know we actually did
                // nothing at all, and exit out.
                if url.is_empty() {
                    drop(data_state);

                    let _msg = req!(context.say("Nothing to queue"));

                    return;
                }
            },
        }
    }

    // Add an entry for the queue (list of song requests) and status
    // (current playing status - None means "nothing"), if there is not
    // already one for each.
    //
    // If these already exist here, nothing is done.
    let _ = state.queue.entry(server_id).or_insert(vec![]);

    drop(state);

    // This will never happen without something actually happening in this
    // run.
    if url.is_empty() {
        return;
    }

    let msg = req!(context.say("Downloading..."));

    let response = match youtube_dl::download(&url) {
        Ok(request) => request,
        Err(Error::YoutubeDL(why)) => {
            let _msg = req!(context.say(why));

            return;
        },
        Err(why) => {
            warn!("impossible: {:?}", why);

            let _msg = req!(context.say("Unknown error downloading song"));

            return;
        },
    };

    let text = format!("Queued **{}** [duration: {}]",
                       response.data.title,
                       get_duration(response.data.duration));

    let mut state = context.music_state.lock().unwrap();

    // Add the song to the `song_completion` map, but _only_ if the two
    // requirements are met:
    //
    // - there is not already a key for the server;
    // - we are in a voice channel in the server.
    let add_to_song_completion = {
        let is_playing = state.status
            .get(&server_id)
            .map_or(false, |status| status.is_some());

        let in_voice = {
            let mut conn = context.conn.lock().unwrap();
            let voice = conn.voice(Some(server_id));
            voice.current_channel().is_some()
        };

        !is_playing && in_voice
    };

    // Add the song to the server's queue, which we make if it doesn't
    // exist.
    {
        let entry = state.queue.entry(server_id).or_insert(vec![]);

        entry.push(MusicRequest {
            response: response,
            requested_in: context.message.channel_id,
            requester_name: context.message.author.name.clone(),
            requester: context.message.author.id,
        });
    }

    // Add this song to the `song_playing`, so that the queue checker will
    // automatically pick it up and try to play the next song in the queue.
    //
    // Setting it to 0 is best here, since no matter what, no sort of timing
    // issue can happen.
    if add_to_song_completion {
        state.song_completion.insert(0, vec![server_id]);
    }

    drop(state);

    let _msg = req!(context.edit(&msg, text));
}

pub fn queue(context: Context) {
    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            warn!("could not find server for channel {}",
                  context.message.channel_id);

            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    let text = {
        let mut temp = String::from("```xl");
        let state = context.music_state.lock().unwrap();

        {
            let requests = match state.queue.get(&server_id) {
                Some(requests) => requests,
                None => {
                    let _msg = req!(context.say("No songs are queued"));

                    return;
                },
            };

            for request in requests {
                temp.push_str(&format!(r#"
- **{}** requested by _{}_ [duration: {}]"#,
request.response.data.title,
request.requester_name,
request.format_duration()));
            }
        }

        drop(state);

        temp.push_str("```");
        temp.truncate(2000);
        temp
    };

    // If there is a key for the server in the queue, but there were no
    // queued requests, then the text will be empty
    if text.is_empty() {
        let _msg = req!(context.say("No songs are queued"));

        return;
    }

    let _msg = req!(context.say(text));
}

pub fn skip(context: Context) {
    let location = req!(get_location(&context));

    if SkipAvailable::find(location).disabled() {
        return;
    }

    let state = context.state.lock().unwrap();
    let (server_id, is_admin) = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => {
            let is_admin = server.permissions_for(
                context.message.channel_id,
                context.message.author.id
            ).contains(permissions::ADMINISTRATOR);

            (server.id, is_admin)
        },
        _ => {
            warn!("could not find server for channel {}",
                  context.message.channel_id);

            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };
    drop(state);

    if is_admin {
        let _msg = req!(context.say("Admin skipped song"));

        let mut state = context.music_state.lock().unwrap();

        for (_k, v) in &mut state.song_completion {
            let removal_index = v.iter()
                .position(|sid| *sid == server_id);

            if let Some(removal_index) = removal_index {
                v.remove(removal_index);

                break;
            }
        }

        state.song_completion.insert(0, vec![server_id]);

        {
            let mut conn = context.conn.lock().unwrap();
            let voice = conn.voice(Some(server_id));
            voice.stop();
        }

        return;
    }

    let err_no = "No song is currently playing";
    let err_already = "You have already voted to skip this song";

    let skip_votes_required = req!(SkipRequired::find(location).as_u64());

    let vote = {
        let mut state = context.music_state.lock().unwrap();

        match state.status.get_mut(&server_id) {
            Some(mut current_opt) => {
                if let Some(mut current) = current_opt.as_mut() {
                    if current.req.requester == context.message.author.id {
                        SkipVote::VoterSkipped
                    } else if !current.skip_votes.contains(&context.message.author.id) {
                        current.skip_votes.push(context.message.author.id);

                        if current.skip_votes.len() >= skip_votes_required as usize {
                            SkipVote::Passed
                        } else {
                            SkipVote::Voted
                        }
                    } else {
                        SkipVote::AlreadyVoted
                    }
                } else {
                    let _msg = req!(context.say(err_no));

                    return;
                }
            },
            _ => {
                let _msg = req!(context.say(err_no));

                return;
            },
        }
    };

    let remove_from_completion = match vote {
        SkipVote::AlreadyVoted => {
            let _msg = req!(context.say(err_already));

            false
        },
        SkipVote::Passed => {
            let mut state = context.music_state.lock().unwrap();
            state.status.insert(server_id, None);
            drop(state);

            {
                let mut conn = context.conn.lock().unwrap();
                let mut voice = conn.voice(Some(server_id));

                voice.stop();
            }

            let _msg = req!(context.say("Skip vote added"));

            true
        },
        SkipVote::Voted => {
            let state = context.music_state.lock().unwrap();

            let current = match state.status.get(&server_id) {
                Some(current_opt) => {
                    if let Some(current) = current_opt.as_ref() {
                        (current.skip_votes.len(), skip_votes_required)
                    } else {
                        let _msg = req!(context.say(err_no));

                        return;
                    }
                },
                _ => {
                    let _msg = req!(context.say(err_no));

                    return;
                },
            };

            drop(state);

            let text = format!("Skip vote added [currently: {}/{}]",
                               current.0,
                               current.1);
            let _msg = req!(context.say(text));

            false
        },
        SkipVote::VoterSkipped => {
            {
                let mut state = context.music_state.lock().unwrap();
                state.status.insert(server_id, None);
            }

            {
                let mut conn = context.conn.lock().unwrap();
                let mut voice = conn.voice(Some(server_id));

                voice.stop();
            }

            let _msg = req!(context.say("Song requester skipped"));

            true
        },
    };

    if remove_from_completion || is_admin {
        let mut state = context.music_state.lock().unwrap();

        for (_k, v) in &mut state.song_completion {
            let removal_index = v.iter()
                .position(|sid| *sid == server_id);

            if let Some(removal_index) = removal_index {
                v.remove(removal_index);

                break;
            }
        }

        state.song_completion.insert(0, vec![server_id]);

        {
            let mut conn = context.conn.lock().unwrap();
            let voice = conn.voice(Some(server_id));
            voice.stop();
        }
    }
}

pub fn status(context: Context) {
    let state = context.state.lock().unwrap();
    let server_id = match state.find_channel(&context.message.channel_id) {
        Some(ChannelRef::Public(server, _channel)) => server.id,
        _ => {
            warn!("could not find server for channel {}",
                  context.message.channel_id);

            let _msg = req!(context.say("Could not find server"));

            return;
        },
    };

    let text = {
        let state = context.music_state.lock().unwrap();
        let current = match state.status.get(&server_id) {
            Some(&Some(ref current)) => current,
            _ => {
                let _msg = req!(context.say("No song is currently playing"));

                return;
            },
        };

        let now = UTC::now().timestamp();
        let ran = now - current.started_at as i64;
        let remaining = (
            current.started_at as i64
            +
            current.req.response.data.duration as i64
            ) - now;

        format!("Playing **{}** [{}/{}] [-{}]",
                current.req.response.data.title,
                get_duration(ran as u64),
                current.req.format_duration(),
                get_duration(remaining as u64))
    };

    req!(context.say(text));
}
