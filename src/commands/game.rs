use crate::sources::safebooru::{self, FileExt};
use crate::state::GameLockContainer;
use crate::utils;
use lazy_static::lazy_static;
use rand::prelude::{SliceRandom, SmallRng};
use rand::SeedableRng;
use regex::Regex;
use serenity::http::Typing;
use serenity::model::channel::ReactionType;
use serenity::prelude::Mentionable;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use std::sync::Arc;
use std::time::Duration;

const IMAGE_COUNT: usize = 9;
const DELAY_BETWEEN_IMAGES: Duration = Duration::from_secs(3);
const DELAY_BETWEEN_LETTERS: Duration = Duration::from_secs(15);
const PLACEHOLDER_CHAR: char = '●';

#[command]
pub async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let typing = Typing::start(ctx.http.clone(), *msg.channel_id.as_u64())?;

    let locks_mutex = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<GameLockContainer>().unwrap()
            .clone()
    };

    let mut locks = locks_mutex.lock().await;

    if locks.contains(&msg.channel_id) {
        drop(locks);
        _ = typing.stop();
        msg.reply(ctx, "There's already an ongoing game in this channel!")
            .await
            .ok();
    } else {
        locks.insert(msg.channel_id);
        drop(locks);

        let mut tags: Vec<&str> = Vec::new();
        // Safebooru added a 2-tag limit to search :(
        // tags.push("status:active");
        // tags.push("order:score");

        // rating:safe doesn't exist anymore?
        if let Ok(channel) = msg.channel_id.to_channel(ctx).await {
            if !channel.is_nsfw() {
                tags.push("rating:general");
            }
        } else {
            tags.push("rating:general");
        }

        let answer_tag = crate::tags::random();
        tags.push(&answer_tag);

        lazy_static! {
            static ref SQ_RE: Regex = Regex::new("[‘’]").unwrap();
            static ref DQ_RE: Regex = Regex::new("[“”]").unwrap();
            static ref NORMAL_RE: Regex = Regex::new(r#"(\s*\([^)]+\))*$"#).unwrap();
        }

        let answer = {
            let sp = answer_tag.replace('_', " ").to_lowercase();
            let sq = SQ_RE.replace_all(&sp, "'");
            let dq = DQ_RE.replace_all(&sq, "\"");

            let normal = NORMAL_RE.replace_all(&dq, "");
            normal.to_string()
        };

        log::debug!(
            "Game started with tag: {} and normalized answer: {}",
            answer_tag,
            answer
        );

        let posts =
            safebooru::posts(tags.join(" ").as_str(), (IMAGE_COUNT * 2).max(100), true).await;

        _ = typing.stop();
        
        if let Ok(posts) = posts {
            let posts: Vec<&safebooru::Post> = posts
                .iter()
                .filter(|post| {
                    post.large_file_url.is_some()
                        && [Some(FileExt::Png), Some(FileExt::Jpg), Some(FileExt::Webp)]
                            .contains(&post.file_ext)
                })
                .collect();

            if posts.len() >= IMAGE_COUNT {
                let answer_clone = answer.clone();
                let posts_clone = posts.clone();
                let game_wait = async move {
                    msg.channel_id
                        .send_message(ctx, |builder| {
                            builder.content("Find the common tag between these images:")
                        })
                        .await
                        .ok();

                    for idx in 0..IMAGE_COUNT {
                        let post = posts_clone[idx];

                        msg.channel_id
                            .send_message(ctx, |builder| {
                                builder.embed(|embed| {
                                    embed.image(post.large_file_url.as_ref().unwrap())
                                })
                            })
                            .await
                            .unwrap();

                        tokio::time::sleep(DELAY_BETWEEN_IMAGES).await;
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;

                    lazy_static! {
                        static ref MASK_RE: Regex = Regex::new(r#"\w"#).unwrap();
                    }

                    let mut mask: Vec<char> = MASK_RE
                        .replace_all(&answer_clone, PLACEHOLDER_CHAR.to_string())
                        .chars()
                        .collect();

                    for _ in 0..(mask.iter().filter(|c| **c == PLACEHOLDER_CHAR).count() - 1) {
                        log::trace!("Mask: {}", mask.iter().collect::<String>());

                        let indices: Vec<usize> = answer_clone
                            .char_indices()
                            .filter_map(|it| {
                                if mask[it.0] == PLACEHOLDER_CHAR {
                                    return Some(it.0);
                                }
                                None
                            })
                            .collect();

                        let reveal = *indices.choose(&mut SmallRng::from_entropy()).unwrap();
                        mask[reveal] = answer_clone.chars().collect::<Vec<char>>()[reveal];

                        msg.channel_id
                            .send_message(ctx, |builder| {
                                builder.content(format!(
                                    "Your hint: **``{}``**",
                                    mask.iter().collect::<String>()
                                ))
                            })
                            .await
                            .unwrap();
                        tokio::time::sleep(DELAY_BETWEEN_LETTERS).await;
                    }

                    msg.channel_id
                        .send_message(ctx, |builder| {
                            builder.content(format!(
                                "Time's up! The answer was **``{}``**",
                                answer_clone
                            ))
                        })
                        .await
                        .unwrap();
                };

                let answer_wait = async move {
                    msg.channel_id
                        .await_reply(ctx)
                        .filter(move |msg| {
                            !msg.author.bot
                                && (utils::alphanumeric(&answer)
                                    .eq_ignore_ascii_case(&utils::alphanumeric(&msg.content))
                                    || answer_tag.eq_ignore_ascii_case(&msg.content))
                        })
                        .timeout(Duration::from_secs(420))
                        .await
                };

                tokio::select! {
                    biased;

                    // Correct answer
                    answer = answer_wait => {
                        match answer {
                            Some(answer) => {
                                answer.reply(ctx, format!("{} answered correctly!", answer.author.mention())).await.ok();
                                answer.react(ctx, ReactionType::Unicode("white_check_mark".to_string())).await.ok();
                                // TODO: award points
                            }
                            None => {
                                msg.reply(ctx, "Timed out waiting for response.").await.ok();
                                return Ok(());
                            }
                        }
                    }
                    // Time's up
                    () = game_wait => {}
                };

                let posts_clone = posts.clone();
                msg.channel_id
                    .send_message(ctx, |builder| {
                        // builder.embed(|tag_embed| {
                        //     tag_embed.title(answer_tag).url(format!(
                        //         "{}/wiki_pages/{}.html",
                        //         safebooru::ENDPOINT,
                        //         "TODO"
                        //     ))
                        // });
                        builder.embed(|posts_embed| {
                            posts_embed.title("Posts").description(
                                posts_clone
                                    .iter()
                                    .take(IMAGE_COUNT)
                                    .map(|post| {
                                        format!(
                                            "{}/posts/{}",
                                            safebooru::ENDPOINT,
                                            if let Some(id) = post.id {
                                                id.to_string()
                                            } else {
                                                "unknown".to_string()
                                            }
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                            )
                        })
                    })
                    .await
                    .ok();
            } else {
                msg.reply(
                    ctx,
                    "No posts were found for the tags I searched for. Try again?",
                )
                .await
                .ok();
            }
        } else {
            log::error!("Error fetching posts: {:?}", posts.unwrap_err());
            msg.reply(ctx, "There was an error fetching posts. Try again later.")
                .await
                .ok();
        }

        let mut locks = locks_mutex.lock().await;
        locks.remove(&msg.channel_id);
    }

    Ok(())
}
