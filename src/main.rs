use std::error::Error;
use url::Url;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
use regex::Regex;
use log::{info, warn, error};
use dotenv::dotenv;
use std::env;
use reqwest;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "사용 가능한 명령어")]
enum Command {
    #[command(description = "이 도움말 표시")]
    Help,
    #[command(description = "봇에 대한 정보")]
    About,
    #[command(description = "URL에서 si 파라미터 제거 테스트")]
    Test(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // .env 파일 로드
    dotenv().ok();
    
    // 로깅 초기화
    pretty_env_logger::init();
    info!("봇 시작중...");

    // .env 파일에서 토큰 가져오기
    let token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN을 .env 파일에 설정해주세요");
    let bot = Bot::new(token);

    info!("FKU-SI 봇 시작됨. 추적 파라미터 제거 활성화.");
    
    // 텔레그램에 명령어 목록 등록
    bot.set_my_commands(Command::bot_commands()).await?;
    info!("텔레그램에 명령어 목록 동기화 완료");

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(commands_handler),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some() && contains_music_link(msg.text().unwrap()))
                .endpoint(handle_music_links),
        );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    
    Ok(())
}

fn contains_music_link(text: &str) -> bool {
    let patterns = vec![
        r"https?://(?:www\.)?youtu(?:\.be|be\.com)/\S+",
        r"https?://(?:www\.)?music\.youtube\.com/\S+",
        r"https?://(?:www\.)?open\.spotify\.com/\S+",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                return true;
            }
        }
    }
    
    false
}

fn remove_si_parameter(url_str: &str) -> String {
    if let Ok(mut url) = Url::parse(url_str) {
        // 쿼리 파라미터 처리
        if url.query().is_some() {
            let query_pairs: Vec<(String, String)> = url.query_pairs()
                .filter(|(k, _)| k != "si")
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            url.set_query(None);
            
            if !query_pairs.is_empty() {
                let query = query_pairs
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<String>>()
                    .join("&");
                
                if !query.is_empty() {
                    url.set_query(Some(&query));
                }
            }
        }
        
        // 특별히 YouTube 짧은 URL 처리
        if url.host_str() == Some("youtu.be") {
            let path = url.path().to_string();
            if path.contains("si=") {
                let new_path = path.split("si=").next().unwrap_or("").trim_end_matches('?');
                url.set_path(new_path);
            }
        }
        
        return url.to_string();
    }
    
    url_str.to_string()
}

fn extract_music_links(text: &str) -> Vec<(String, String)> {
    let patterns = vec![
        r"(https?://(?:www\.)?youtu(?:\.be|be\.com)/\S+)",
        r"(https?://(?:www\.)?music\.youtube\.com/\S+)",
        r"(https?://(?:www\.)?open\.spotify\.com/\S+)",
    ];
    
    let mut links = Vec::new();
    
    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(text) {
                if let Some(m) = cap.get(1) {
                    let original_url = m.as_str();
                    let cleaned_url = remove_si_parameter(original_url);
                    
                    if original_url != cleaned_url {
                        links.push((original_url.to_string(), cleaned_url));
                    }
                }
            }
        }
    }
    
    links
}

async fn commands_handler(bot: Bot, msg: Message, cmd: Command) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cmd {
        Command::Help => {
            let help_text = format!(
                "{}\n\n\
                 봇 동작 방식:\n\
                 1. 그룹에서 봇에 관리자 권한이 있는 경우:\n   \
                    - YouTube, YouTube Music, Spotify 링크가 포함된 메시지를 감지\n   \
                    - 원본 메시지를 삭제\n   \
                    - 사용자 닉네임과 함께 si 파라미터가 제거된 링크를 새 메시지로 전송\n\n\
                 2. 그룹에서 봇에 관리자 권한이 없는 경우:\n   \
                    - YouTube, YouTube Music, Spotify 링크가 포함된 메시지를 감지\n   \
                    - 원본 메시지에 답장으로 인라인 버튼 형태의 정리된 링크 제공",
                Command::descriptions().to_string()
            );
            
            bot.send_message(msg.chat.id, help_text).await?;
        }
        Command::About => {
            bot.send_message(
                msg.chat.id, 
                "YouTube, YouTube Music, Spotify 링크에서 추적 파라미터(si)를 제거해주는 봇입니다."
            ).await?;
        }
        Command::Test(url) => {
            if url.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    "URL을 입력해주세요. 예: /test https://youtu.be/Vc-ByDGOuQE?si=qIy-ihfrRKmDAPZP"
                ).await?;
                return Ok(());
            }
            
            let cleaned_url = remove_si_parameter(&url);
            
            if url == cleaned_url {
                bot.send_message(
                    msg.chat.id,
                    format!("제거할 추적 파라미터가 없습니다.\n\n원본: {}", url)
                ).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    format!("추적 파라미터가 제거되었습니다.\n\n원본: {}\n\n정리됨: {}", url, cleaned_url)
                ).await?;
            }
        }
    }
    
    Ok(())
}

async fn handle_music_links(bot: Bot, msg: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text = msg.text().unwrap();
    let links = extract_music_links(text);
    
    if links.is_empty() {
        return Ok(());
    }
    
    // 봇이 관리자 권한을 가지고 있는지 확인
    let chat_member = match bot
        .get_chat_member(msg.chat.id, bot.get_me().await?.id)
        .await {
            Ok(member) => member,
            Err(e) => {
                error!("관리자 권한 확인 중 오류 발생: {:?}", e);
                return handle_without_admin_rights(&bot, &msg, &links).await;
            }
        };
    
    let is_admin = chat_member.kind.is_privileged();
    
    if is_admin {
        handle_with_admin_rights(&bot, &msg, &links).await
    } else {
        handle_without_admin_rights(&bot, &msg, &links).await
    }
}

async fn handle_with_admin_rights(bot: &Bot, msg: &Message, links: &[(String, String)]) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 원본 메시지 삭제
    if let Err(e) = bot.delete_message(msg.chat.id, msg.id).await {
        warn!("메시지 삭제 실패: {:?}", e);
        return handle_without_admin_rights(bot, msg, links).await;
    }
    
    // 사용자 이름 가져오기
    let username = if let Some(user) = msg.from() {
        if let Some(username) = &user.username {
            username.to_string()
        } else {
            user.first_name.clone()
        }
    } else {
        "Unknown".to_string()
    };
    
    // 정리된 링크 전송
    let mut cleaned_text = msg.text().unwrap().to_string();
    
    for (original, cleaned) in links {
        cleaned_text = cleaned_text.replace(original, cleaned);
    }
    
    bot.send_message(
        msg.chat.id, 
        format!("{}: {}", username, cleaned_text)
    ).await?;
    
    Ok(())
}

async fn handle_without_admin_rights(bot: &Bot, msg: &Message, links: &[(String, String)]) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 관리자 권한이 없는 경우 인라인 버튼으로 답장
    let mut keyboard = Vec::new();
    
    for (i, (_, cleaned)) in links.iter().enumerate() {
        // reqwest::Url로 변환
        match reqwest::Url::parse(cleaned) {
            Ok(url) => {
                let row = vec![InlineKeyboardButton::url(
                    format!("정리된 링크 #{}", i+1),
                    url,
                )];
                keyboard.push(row);
            },
            Err(e) => {
                warn!("URL 파싱 오류: {}, URL: {}", e, cleaned);
            }
        }
    }
    
    // 버튼이 없으면 처리하지 않음
    if keyboard.is_empty() {
        return Ok(());
    }
    
    let markup = InlineKeyboardMarkup::new(keyboard);
    
    bot.send_message(msg.chat.id, "추적 파라미터가 제거된 링크:")
        .reply_to_message_id(msg.id)
        .reply_markup(markup)
        .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_si_parameter_youtube() {
        let original = "https://youtu.be/Vc-ByDGOuQE?si=qIy-ihfrRKmDAPZP";
        let expected = "https://youtu.be/Vc-ByDGOuQE";
        assert_eq!(remove_si_parameter(original), expected);
    }

    #[test]
    fn test_remove_si_parameter_youtube_music() {
        let original = "https://music.youtube.com/watch?v=nmYDYalgb5w&si=GGi18ac_fxnx4F1b";
        let expected = "https://music.youtube.com/watch?v=nmYDYalgb5w";
        assert_eq!(remove_si_parameter(original), expected);
    }

    #[test]
    fn test_remove_si_parameter_spotify() {
        let original = "https://open.spotify.com/track/1FYWnRofuIgJf62AnX8i5S?si=bf00147df50f4141";
        let expected = "https://open.spotify.com/track/1FYWnRofuIgJf62AnX8i5S";
        assert_eq!(remove_si_parameter(original), expected);
    }

    #[test]
    fn test_remove_si_parameter_with_multiple_params() {
        let original = "https://music.youtube.com/watch?v=nmYDYalgb5w&si=GGi18ac_fxnx4F1b&list=RDAMVMnmYDYalgb5w";
        let expected = "https://music.youtube.com/watch?v=nmYDYalgb5w&list=RDAMVMnmYDYalgb5w";
        assert_eq!(remove_si_parameter(original), expected);
    }
}
