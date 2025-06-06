use std::sync::Arc;
use teloxide::{prelude::*, types::Update};

mod config;
mod database;
mod modules;
use config::Config;
use database::Database;
use modules::{ModuleManager, pig_game::PigGameModule, powerful_nahruk::PowerfulNahrukModule, reactions::ReactionsModule};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let config = Config::load_or_default();
    log::info!("Config loaded: {:?}", config);


    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .or_else(|| config.database_url.clone())
        .expect("DATABASE_URL is not set in environment or config");

    let db = Arc::new(
        Database::connect(&database_url)
            .await
            .expect("Failed to connect to database"),
    );

    log::info!("Running database migrations...");
    db.migrate().await.expect("Failed to run migrations");
    log::info!("Database migrations completed");

    let mut module_manager = ModuleManager::new();
    module_manager.register_module(Box::new(PigGameModule::new()));
    module_manager.register_module(Box::new(PowerfulNahrukModule::new()));
    module_manager.register_module(Box::new(ReactionsModule::new()));
    let module_manager = Arc::new(module_manager);

    let config = Arc::new(config);

    let token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    let bot = Bot::new(token);

    let handler = Update::filter_message().endpoint(handle_message);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![module_manager, db, config])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    module_manager: Arc<ModuleManager>,
    db: Arc<Database>,
    config: Arc<Config>
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            let parts: Vec<&str> = text[1..].split_whitespace().collect();
            if let Some(command) = parts.first() {
                let args = parts.get(1..).unwrap_or(&[]).to_vec();

                match *command {
                    "help" => {
                        let commands = module_manager.get_all_commands();
                        let help_text = if commands.is_empty() {
                            "No commands available".to_string()
                        } else {
                            commands.join("\n")
                        };
                        bot.send_message(
                            msg.chat.id,
                            format!("Available commands:\n{}", help_text),
                        )
                        .await?;
                        return Ok(());
                    }
                    _ => {
                        if module_manager
                            .handle_command(bot.clone(), msg.clone(), command, args, &db, &config)
                            .await?
                        {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    module_manager.handle_message(bot, msg, &db, &config).await?;
    Ok(())
}
