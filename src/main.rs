#![allow(unused_must_use, unused_unsafe, unused_mut, unused_parens, unused_imports, unused_variables)]

use std::{env, process::Command};

use lazy_static::lazy_static; // 1.4.0
use std::sync::Mutex;

use std::collections::HashMap;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};


use std::convert::TryFrom;

use serenity::model::channel::ReactionType;
use serenity::model::id::EmojiId;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::io::Write;
use std::path::Path;

use rand::prelude::*;

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs;

lazy_static! {

	static ref GAMES : Mutex<HashMap<u64, Game>> = Mutex::new(HashMap::new());

}

lazy_static! {
	static ref HIGHERLOWERS : Mutex<HashMap<u64, HigherLower>> = Mutex::new(HashMap::new());
}

pub struct HigherLower {
	pub prev: u8,
	pub streak: u64
}


struct Handler;

#[derive(Debug)]
struct Game {

	pub word : String,
	pub finished : bool,
	pub guess : Vec<char>,
	pub consecutive_false : u64

}

#[derive(Debug)]
pub enum Errors {
	NoGame,
	InvalidGuess,
	GameFinished
}

#[derive(Debug)]
pub enum GameState {
	Done(String),
	Correct(String, Vec<char>, bool),
	Wrong(String, Vec<char>),
	CorrectNew(String, Vec<char>, char),
	GotIt
}

impl Game {

	pub fn new(channel_id : &u64) -> Result<(String, Vec<char>), ()> {

		let mut flag = false;

		let mut str_version : String = String::new(); 

		while ! flag {
			let command = Command::new("shuf")
        	.arg("-n1")
        	.arg("/usr/share/dict/words")
        	.output()
        	.expect("no")
        	.stdout;

        	str_version = std::str::from_utf8(&command).unwrap().trim().to_lowercase().to_string();

        	flag = str_version.to_string().chars().all(|x| x.is_alphabetic());
		}

        println!("New word is : {}", str_version);


        let mut map = GAMES.lock().unwrap(); 


        let game = Game {
        		word : str_version.to_string(),
        		finished : false,
        		guess : Vec::new(),
        		consecutive_false : 0
        	};

        if let Some(game) = map.get(channel_id) {
        	if ! game.finished {
        		println!("Game already exists for this channel");
        		return Err(())
        	}
        } 

        let returned = map.insert(*channel_id, game);

        Ok((str_version.trim().to_string(), Vec::new()))

	}

	pub fn guess(channel_id : &u64, guess: &str) -> Result<GameState, Errors> {


			let mut map = GAMES.lock().unwrap();

    		let game : &Game = match map.get(channel_id) {
    			None => {return Err(
    				Errors::NoGame
    			);}

    			Some(game) => game
    		};

    		if game.finished {

    			return Err(
    				Errors::GameFinished
    			);

    		}

    		let vec_of_guess = game.guess.to_vec();
    		let word = game.word.to_string();

    		drop(game);
    		drop(map);


    		// let mut map = GAMES.lock().unwrap();

    		let char_vec: Vec<char> = guess.chars().collect();

    		let mut map = GAMES.lock().unwrap();

    		if ! (char_vec.len() == 1) || ! (char_vec[0].is_alphabetic())  {
    			if guess == word {
    				let game = Game {
    					word: word.to_string(),
    					finished : true, 
    					guess: Vec::new(),
    					consecutive_false: 0,
    				};
    				map.insert(*channel_id, game);

    				return Ok(GameState::GotIt);
    			} else {}
    			return Err(
    				Errors::InvalidGuess
    			);
    		} 

    		let char_to_guess = char_vec[0];
			
			if word.chars().collect::<Vec<char>>().contains(&char_to_guess) {
				if vec_of_guess.contains(&char_to_guess) {
					let mut flag = true;

					for ch in word.chars().collect::<Vec<char>>() {
						if !(vec_of_guess.contains(&ch)) {
							flag = false;
						}
					}

					return Ok(GameState::Correct(word.to_string(), vec_of_guess.to_vec(), flag));
				} else {
					let mut new_vec : Vec<char> = vec_of_guess.to_vec();
					new_vec.push(char_to_guess);
					
					let mut flag = true;

					for ch in word.chars().collect::<Vec<char>>() {
						if !(new_vec.contains(&ch)) {
							flag = false;
						}
					}

					let replacement = Game {
						word : word.to_string(), 
						guess: new_vec.to_vec(),
						finished: flag,
						consecutive_false: 0
					};

					map.insert(*channel_id, replacement);

					return Ok(GameState::Correct(word.to_string(), new_vec, flag));

				}
			} else {
				return Ok(GameState::Wrong(word.to_string(), vec_of_guess.to_vec()));
			}
	}

}



impl HigherLower {
	fn play_higher_lower(id : u64) -> u8 {

		let mut map = HIGHERLOWERS.lock().unwrap();

		println!("HERE? ");
		let starting_value : u8 = rand::random();

		let game_state = HigherLower {
			streak: 0,
			prev: starting_value,
		};

		map.insert(id, game_state);

		return starting_value;

	}

	fn higher(id : u64) -> Result<((bool, u8, u64)), ()>{
		let new_value : u8 = rand::random();

		let mut map = HIGHERLOWERS.lock().unwrap();

		if let Some(state) = map.get(&id) {

			let prev_state : u8 = state.prev; 
			let prev_score : u64 = state.streak;

			println!("NEW NUMBER : {} <> OLD NUMBER : {}", new_value, prev_state);

			drop(map);

			let flag = new_value > prev_state; 

			let mut map = HIGHERLOWERS.lock().unwrap();

			if flag {

				map.insert(id, HigherLower {
					prev : new_value,
					streak : prev_score + 1,
				});

				return Ok((flag, new_value, prev_score + 1));

			} else {

				map.insert(id, HigherLower {
					prev : new_value,
					streak : 0
				});
				return Ok((flag, new_value, 0));
			}

			

		} else {
			return Err(());
		}
	}

	fn lower(id : u64) -> Result<((bool, u8, u64)), ()>{
		let new_value : u8 = rand::random();

		let mut map = HIGHERLOWERS.lock().unwrap();

		if let Some(state) = map.get(&id) {

			let prev_state : u8 = state.prev; 
			let prev_score : u64 = state.streak;

			println!("NEW NUMBER : {} <> OLD NUMBER : {}", new_value, prev_state);

			drop(map);

			let flag = new_value <= prev_state; 

			let mut map = HIGHERLOWERS.lock().unwrap();

			if flag {

				map.insert(id, HigherLower {
					prev : new_value,
					streak : prev_score + 1,
				});

				return Ok((flag, new_value, prev_score + 1));

			} else {

				map.insert(id, HigherLower {
					prev : new_value,
					streak : 0
				});
				return Ok((flag, new_value, 0));
			}

			

		} else {
			return Err(());
		}
	}
}

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {

    	println!("Received Message from {} : {}", msg.author.name, msg.content);




    	if msg.content == "!higherlower" {
    		
    		println!("Playing higherlower");
    		let init = HigherLower::play_higher_lower(msg.author.id.0);
    		msg.reply(&ctx.http, &format!("Starting new higher/lower game for {} \n Current Score : 0 \n Starting Value = {}", msg.author.name, init)).await;

    	}

    	if msg.content == ("!higher") {
    		match HigherLower::higher(msg.author.id.0) {

    			Err(_) => {
    				msg.reply(&ctx.http, "No game in session for you. Play with !higherlower").await;
    			},

    			Ok((true, new_value, streak)) => {

    				msg.reply(&ctx.http, &format!("Correct! Number was {}. Streak now at {}", new_value, streak)).await;

    			},

    			Ok((false, new_value, streak)) => {
    				msg.reply(&ctx.http, &format!("f. Number was {}. Score now 0.", new_value)).await;
    			}

    		}
    	}

    	if msg.content == ("!lower") {
    		match HigherLower::lower(msg.author.id.0) {

    			Err(_) => {
    				msg.reply(&ctx.http, "No game in session for you. Play with !higherlower").await;
    			},

    			Ok((true, new_value, streak)) => {

    				msg.reply(&ctx.http, &format!("Correct! Number was {}. Streak now at {}", new_value, streak)).await;

    			},

    			Ok((false, new_value, streak)) => {
    				msg.reply(&ctx.http, &format!("f. Number was {}. Score now 0.", new_value)).await;
    			}

    		}
    	}

    	if msg.content == "!photo" {
    		let paths = vec!["./src/resources/cow.jpg"];

    		let files = fs::read_dir("./src/resources/").unwrap();

    		let mut vec : Vec<String> = Vec::new();

    		for f in files {
    			let x = f.unwrap().path().display().to_string();
    			vec.push(x);
    		}

    		unsafe {

    			let mut rng = thread_rng();

				vec.shuffle(&mut rng);
    		}


    		let new_vec = vec![vec[0].as_str()];
    		println!("{:#?}",  new_vec);
    		// let pwd = Command::new("pwd")
    		// .output()
    		// .expect("sgdsg")
    		// .stdout;

    		// println!("{}", std::str::from_utf8(&pwd).unwrap());

			let _ = msg.channel_id.send_files(&ctx.http, new_vec, |m| {
    			m.content("")
			}).await.unwrap();
    	}

    	if msg.content == "!fortune" {

    		println!("Command called");
    		let fortune_command = Command::new("fortune")
    			.arg("-o")
    			.output()
    			.expect("Uh oh")
    			.stdout;

    		let mut string_version = std::str::from_utf8(&fortune_command).unwrap();

			let y : u8 = rand::random();

    		if y < 20 {
    			string_version = r#"Dream pushed Hasan against a wall, sneering, "I hate you."

Hasan scoffed. "I hate you too." Their eyes glanced down to Dream’s lips, and Dream smirked.

"Like what you see?"

Hasan flushed, twisting their head. "As if.""#;
    		}

    		let cow = Command::new("cowsay")
    			.arg(string_version)
    			.output()
    			.expect("sdf")
    			.stdout;

    		let cow_text = std::str::from_utf8(&cow).unwrap();



    		msg.channel_id.say(&ctx.http, &format!("The Deformed Cow Says: \n `{}`", cow_text)).await;
    	}

    	if msg.content == "!flip" {
    		if rand::random() {
    			msg.reply(&ctx.http, "Heads").await;
			} else {
				msg.reply(&ctx.http, "Tails").await;
			}
    	}

    	if msg.content == "!hangman" {

    		if let Ok((string, guessed_chars)) = Game::new(&msg.channel_id.0) {
    			let mut place_holder = String::new();

    			for x in string.chars() {
    				place_holder.push_str("_");
    				place_holder.push_str(" ");
    			}

    			msg.reply(&ctx.http, &format!("{} started a hangman game: \n `{}` ({} characters)", msg.author.name, place_holder, string.len())).await;
    		} else {
    			msg.reply(&ctx.http, "Starting a hangman game when there's one already active is a bannable offence.").await;
    		}
    		
  
    	}

    	if msg.content.starts_with("!guess") {
    		
    		let mut x = msg.content.split(" ");

    		let str_vec : Vec<&str> = x.collect(); 

    		if !(str_vec.len() == 2) {
    			msg.reply(&ctx.http, "Invalid guess. Must be alphabetic and only one character.").await;
    			return;
    		}



    		match Game::guess(&msg.channel_id.0, str_vec.last().unwrap()) {
    			Err(Errors::NoGame) => {
    				msg.reply(&ctx.http, "No active game").await;
    			},
    			Err(Errors::InvalidGuess) => {
    				msg.reply(&ctx.http, "Invalid guess. Must be alphabetic and only one character.").await;
    			},
    			Err(Errors::GameFinished) => {
    				msg.reply(&ctx.http, "No active game.").await;
    			}
    			Ok(GameState::GotIt) => {

    				msg.reply(&ctx.http, "CORRECT! Game done").await;

    			}
    			Ok(GameState::Correct(string, y, done)) => {

    				// let mut map = GAMES.lock().unwrap();

    				// println!("{} {:#?}", string, y );

    				let mut so_far = String::new();

    				for x in string.chars() {
    					if y.contains(&x) {
    						so_far.push(x);
    					} else {
    						so_far.push_str("_");
    					}
    					so_far.push_str(" ");
    				}

    				if done {
    					msg.reply(&ctx.http, &format!("correct. game complete \n `{}`", string)).await;
    				} else {
    					msg.reply(&ctx.http, &format!("correct. Guess so far: \n `{}` ({} characters)", so_far, string.len())).await;
    				}
    			},

    			Ok(GameState::CorrectNew(string, z, cha)) =>  {

    				let mut y = z.to_vec();

    				y.push(cha); 

    				let mut so_far = String::new();

    				for x in string.chars() {
    					if y.contains(&x) {
    						so_far.push(x);
    					} else {
    						so_far.push_str("_");
    					}
    					so_far.push_str(" ");
    				}

    				msg.reply(&ctx.http, &format!("Correct! Guess now : \n `{}` ({} characters)", so_far, string.len())).await;

    			}

    			Ok(GameState::Wrong(string, y)) => {
    				// println!("{} {:#?}", string, y );

    				let mut so_far = String::new();

    				for x in string.chars() {
    					if y.contains(&x) {
    						so_far.push(x);
    					} else {
    						so_far.push_str("_");
    					}
    					so_far.push_str(" ");
    				}

    				msg.reply(&ctx.http, &format!("Guess doesn't match. Try again. \n `{}` ({} characters)", so_far, string.len())).await;
    			},
    			Ok(GameState::Done(string)) => {
    				msg.reply(&ctx.http, "SOLVED!").await;
    			}

    		}

    		
    	}

    	if msg.content == "!talktome" {
    		let _ = msg.channel_id.send_message(&ctx.http, |m| {

    			let fortune_command = Command::new("fortune")
    				.arg("-o")
    				.output()
    				.expect("Uh oh")
    				.stdout;

    			let mut string_version = std::str::from_utf8(&fortune_command).unwrap();

    			m.content(string_version);
    			m.tts(true);

    			m.embed(|mut e| {
        		e.title("The Deformed Cow has Sent a Voice Message");
        		e.description(&format!("Courtesy of {}", msg.author.name));
        			e
    			});

    			m
				}).await;
    	}

    	let emote_flag : u8 = rand::random();

    	if emote_flag < 40 {
    		let mut vec : Vec<&str> = vec![
    				 "<:WhatChamp:749078823802503279>",
    				 "<:Pog:464354871445946378>",
    				 "<:pepepains:596439009702445056>",
    				 "<:JoeWeird:727319067630895176>",
    				 "<:OMEGALUL:464355555377545216>",
    				 "<:PepeHands:464354349880180756>",
    				 "<:LULW:643479275319590913>",
    				 "<:YEP:662016139769151509>",
    				 "<:hmm:748626592015843359>",
    				 "<:jebaited:671853396260552714>",
    				 "<:suicide:464354831801253898>"
    			];

    		unsafe {

    			let mut rng = thread_rng();

				vec.shuffle(&mut rng);
    		}

    		let x : ReactionType = ReactionType::try_from(vec[0].to_string()).unwrap();

    		match msg.reply(&ctx.http, vec[0]).await {
    			Ok(_) => {println!("SENT EMOJI REPLY");}
	    		Err(why) => {println!("bope {:?}", why);}
    		}


    	}

    	if msg.content == "!alarm" {
    		msg.channel_id.say(&ctx.http, ":alarm_clock:").await;
    	}

    	if msg.attachments.len() > 0 {
    		println!("Contains an attachment, attempting to download.");
    	}

    	for attachment in msg.attachments {
    		let content = match attachment.download().await {
    			Ok(content) => content,
    			Err(why) => {
    				println!("Failed downloading attachment: {}", why);
    				return;
    			}
    		};

    		let mut file = match File::create(&format!("./src/resources/{}", &attachment.filename)).await {
    			Ok(file) => file,
    			Err(why) => {
    				println!("Could not create file : {}", why);

    				return;
    			}
    		};

    		if let Err(why) = file.write(&content).await {
    			println!("Error writing to file {}", why);

    			return;
    		}
    	}
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        
    }
}

#[tokio::main]
async fn main() {

    let token = "ODUxNzk4MTEyMzgxMzA0ODcz.YL9gzw.7dYcwpIWHfXGKodbIOjA_caGEmY";


    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");


    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}