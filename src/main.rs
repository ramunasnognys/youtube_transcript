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
    // This method formats the timestamp of a transcript item into a readable string
    // It takes the start time in seconds and converts it to [MM:SS] format
    // For example: 
    // - If start time is 65.0 seconds, returns "[01:05]"
    // - If start time is 125.5 seconds, returns "[02:05]"
    fn format_time(&self) -> String {
        let start_mins = (self.start / 60.0).floor(); // Convert seconds to minutes
        let start_secs = (self.start % 60.0).floor(); // Get remaining seconds
        format!("[{:02}:{:02}]", start_mins, start_secs) // Format as [MM:SS]
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
    // First convert TranscriptItems to the format we need
    let content = transcript.iter()
        .map(|item| format!("{} {}", item.format_time(), item.text))
        .collect::<Vec<_>>()
        .join("\n");

    // Normalize the timestamps
    let normalized = normalize_timestamps(&content);

    // Save the normalized version
    fs::write(format!("transcript_{}.txt", video_id), normalized)?;
    Ok(())
}

fn process_timestamp_line(line: &str) -> Option<(f64, String)> {
    if let Some(timestamp_end) = line.find(']') {
        if line.starts_with('[') {
            let timestamp_str = &line[1..timestamp_end];
            let text = line[timestamp_end + 1..].trim().to_string();
            
            // Convert timestamp to seconds
            if let Some((minutes, seconds)) = timestamp_str.split_once(':') {
                if let (Ok(min), Ok(sec)) = (minutes.parse::<f64>(), seconds.parse::<f64>()) {
                    return Some((min * 60.0 + sec, text));
                }
            }
        }
    }
    None
}

// Normalize timestamps
#[allow(unused_mut)]
fn normalize_timestamps(content: &str) -> String {
    let mut normalized = String::new();
    let mut current_timestamp = 0;
    let interval = 6; // 6-second intervals
    
    // Process each line and collect timestamps and text
    let mut entries: Vec<(f64, String)> = content
        .lines()
        .filter_map(process_timestamp_line)
        .collect();
    
    // Sort by timestamp if needed
    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    // Group text into 6-second intervals
    while current_timestamp <= (entries.last().map(|e| e.0).unwrap_or(0.0) as i32) {
        let start_time = current_timestamp as f64;
        let end_time = (current_timestamp + interval) as f64;
        
        let text: String = entries
            .iter()
            .filter(|(ts, _)| *ts >= start_time && *ts < end_time)
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        if !text.is_empty() {
            let minutes = current_timestamp / 60;
            let seconds = current_timestamp % 60;
            normalized.push_str(&format!("[{}:{:02}] {}\n", minutes, seconds, text));
        }
        
        current_timestamp += interval;
    }
    
    normalized
}
// Timstamp line end


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // First, let's normalize any existing transcripts if specified
    if let Ok(content) = std::fs::read_to_string("transcript_RcYjXbSJBN8.txt") {
        println!("Normalizing existing transcript...");
        let normalized = normalize_timestamps(&content);
        println!("Normalized transcript:");
        println!("{}", normalized);
        // Optionally save the normalized version
        fs::write("transcript_RcYjXbSJBN8_normalized.txt", normalized)?;
    }

    // Then proceed with the original main function logic
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