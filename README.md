# YouTube Transcript Downloader

A Rust-based command-line tool for downloading and formatting transcripts from YouTube videos.

## Overview

This tool provides an easy way to download and format transcripts from YouTube videos, saving them in a readable format with timestamps. It's particularly useful for content creators, researchers, or anyone needing text versions of YouTube content.

## Features

- Downloads transcripts directly from YouTube videos
- Formats timestamps in a readable [MM:SS] format
- Handles HTML entities in the transcript text
- Configurable through a simple JSON config file
- Robust error handling and user feedback
- Clean, formatted output saved to text file

## Prerequisites

- Rust toolchain (cargo, rustc)
- Internet connection
- Valid YouTube video URL/ID

## Installation

1. Clone the repository:
```bash
git clone [repository-url]
cd youtube_transcript
```

2. Build the project:
```bash
cargo build --release
```

The executable will be available in `target/release/youtube_transcript`

## Configuration

Create a `config.json` file in the project root with the following structure:

```json
{
    "video_url": "https://www.youtube.com/watch?v=YOUR_VIDEO_ID",
    "video_id": "YOUR_VIDEO_ID"
}
```

You only need to provide either the video URL or ID - the program will extract the ID from the URL if needed.

## Usage

1. Update the `config.json` with your desired YouTube video URL or ID
2. Run the program:
```bash
./target/release/youtube_transcript
```

The transcript will be saved as `transcript_[VIDEO_ID].txt` in the current directory.

## Output Format

The transcript is saved in a clean, readable format with timestamps:
```
[00:00] First line of transcript
[00:02] Second line of transcript
[00:05] And so on...
```

## Technical Details

### Dependencies

- tokio (async runtime)
- reqwest (HTTP client)
- serde (JSON serialization)
- serde_json (JSON parsing)
- regex (transcript parsing)
- html-escape (HTML entity decoding)

### Main Components

- `TranscriptItem`: Struct for holding individual transcript entries
- `Config`: Struct for parsing configuration file
- `get_transcript`: Main function for fetching and parsing transcripts
- `save_transcript`: Function for formatting and saving output

### Error Handling

The program includes comprehensive error handling for:
- Network issues
- Invalid video IDs
- Missing captions
- Parsing errors
- File I/O errors

## Limitations

- Only works with videos that have available captions
- Currently only downloads the first available caption track
- Requires a stable internet connection

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Please refer to the repository's license file for licensing information.

## Acknowledgments

Special thanks to all contributors and the Rust community for the excellent crate ecosystem.