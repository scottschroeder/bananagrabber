use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    model::{
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            let dm = msg
                .author
                .dm(&ctx, |m| m.content("You have requested ping!"))
                .await;

            if let Err(why) = dm {
                log::error!("Error when direct messaging user: {:?}", why);
            }

            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                log::error!("Error sending message: {:?}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                "id" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected user option")
                        .resolved
                        .as_ref()
                        .expect("Expected user object");

                    if let ApplicationCommandInteractionDataOptionValue::User(user, _member) =
                        options
                    {
                        format!("{}'s id is {}", user.tag(), user.id)
                    } else {
                        "Please provide a valid user".to_string()
                    }
                }
                "bananagrabber" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected user option")
                        .resolved
                        .as_ref()
                        .expect("Expected user object");

                    if let ApplicationCommandInteractionDataOptionValue::String(s) = options {
                        match crate::media_extraction::fetch_url_through_cross_posts(s).await {
                            Ok(u) => match u {
                                Some(m) => m,
                                // None => "could not find media".to_string(),
                                None => s.clone(),
                            },
                            Err(e) => {
                                log::error!("error while looking up url: {}", e);
                                s.clone()
                            }
                        }
                    } else {
                        "please provide a url".to_string()
                    }
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                log::error!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                // .create_application_command(|command| {
                //     command.name("ping").description("A ping command")
                // })
                // .create_application_command(|command| {
                //     command
                //         .name("id")
                //         .description("Get a user id")
                //         .create_option(|option| {
                //             option
                //                 .name("id")
                //                 .description("The user to lookup")
                //                 .kind(ApplicationCommandOptionType::User)
                //                 .required(true)
                //         })
                // })
                .create_application_command(|command| {
                    command
                        .name("bananagrabber")
                        .description("Extract the media out of a reddit link")
                        .create_option(|option| {
                            option
                                .name("url")
                                .description("The reddit link")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
        })
        .await;

        // for x in ApplicationCommand::get_global_application_commands(&ctx.http).await {
        //     log::info!("global app: {:#?}", x);
        // }

        // ApplicationCommand::delete_global_application_command(&ctx.http, 939674884773150770.into()).await;

        log::debug!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

pub async fn bot_start() -> anyhow::Result<()> {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    //
    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    Ok(client.start().await?)
}
