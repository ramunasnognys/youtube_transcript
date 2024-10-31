use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    video_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranscriptItem {
    text: String,
    start: f64,
    duration: f64,
}

impl TranscriptItem {
    fn format_time(&self) -> String {
        let start_mins = (self.start / 60.0).floor();
        let start_secs = (self.start % 60.0).floor();
        format!("[{:02}:{:02}]", start_mins, start_secs)
    }
}

fn extract_json(html: &str) -> Option<&str> {
    let start_marker = "ytInitialPlayerResponse = ";
    let end_marker = ";</script>";

    html.find(start_marker)
        .map(|start_idx| {
            let start_pos = start_idx + start_marker.len();
            let sub_str = &html[start_pos..];
            let end_pos = sub_str.find(end_marker).unwrap_or(sub_str.len());
            &sub_str[..end_pos]
        })
}

fn build_youtube_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}

async fn get_transcript(video_id: &str) -> Result<Vec<TranscriptItem>, Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let url = build_youtube_url(video_id);
    println!("Fetching video page...");
    
    let response = client
        .get(&url)
        .send()
        .await?;

    let html = response.text().await?;

    println!("Extracting caption data...");
    let json_str = extract_json(&html).ok_or("Cannot find player data")?;
    
    let parsed: serde_json::Value = serde_json::from_str(json_str)?;

    if let Some(captions) = parsed
        .get("captions")
        .and_then(|c| c.get("playerCaptionsTracklistRenderer"))
        .and_then(|p| p.get("captionTracks"))
        .and_then(|t| t.as_array())
    {
        println!("Found caption tracks...");
        if let Some(first_track) = captions.first() {
            if let Some(base_url) = first_track.get("baseUrl").and_then(|u| u.as_str()) {
                println!("Downloading transcript...");
                let transcript_response = client.get(base_url).send().await?;
                let transcript_xml = transcript_response.text().await?;

                println!("Parsing transcript data...");
                let re = regex::Regex::new(r#"<text start="([^"]+)" dur="([^"]+)"[^>]*>([^<]+)</text>"#)?;
                let mut transcript = Vec::new();

                for cap in re.captures_iter(&transcript_xml) {
                    let start: f64 = cap[1].parse()?;
                    let duration: f64 = cap[2].parse()?;
                    let text = html_escape::decode_html_entities(&cap[3]).into_owned();

                    transcript.push(TranscriptItem {
                        text,
                        start,
                        duration,
                    });
                }

                if transcript.is_empty() {
                    return Err("No transcript lines found in the response".into());
                }

                println!("Successfully parsed {} lines", transcript.len());
                return Ok(transcript);
            }
        }
    }

    Err("No captions found for this video".into())
}

fn save_transcript(transcript: &[TranscriptItem], video_id: &str) -> Result<(), Box<dyn Error>> {
    let output = transcript.iter()
        .map(|item| format!("{} {}", item.format_time(), item.text))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(format!("transcript_{}.txt", video_id), output)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Read config file
    let config_text = fs::read_to_string("config.json")
        .expect("Failed to read config.json. Make sure it exists in the project root.");
    
    let config: Config = serde_json::from_str(&config_text)?;
    
    println!("Starting transcript download for video ID: {}", config.video_id);
    
    match get_transcript(&config.video_id).await {
        Ok(transcript) => {
            println!("\nTranscript found! ({} lines)\n", transcript.len());
            
            // Save to file
            save_transcript(&transcript, &config.video_id)?;
            println!("\nTranscript saved to transcript_{}.txt", config.video_id);

            // Display on console
            for item in transcript {
                println!("{} {}", item.format_time(), item.text);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}