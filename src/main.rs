extern crate rayon;
use rayon::prelude::*;

use std::fs::{remove_file, rename, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let link = File::open("link.txt")?;
    let reader = BufReader::new(link);

    // Youtube download
    for line in reader.lines() {
        let line = line?;
        println!("Downloading {}...", line);
        let mut cmd = youtube_default();
        cmd.arg(line);
        cmd.output().expect("Can't download link");
    }

    println!("Download completed!");

    let mut mp4_list: Vec<String> = Vec::new();
    let mut jpg_list: Vec<String> = Vec::new();

    // Find mp4 files
    for file in std::fs::read_dir("temp_mp4_dir")? {
        let entry = file?;
        let path = entry.path();
        if !path.is_dir() {
            match entry.file_name().into_string() {
                Ok(path) if path.contains(".mp4") => {
                    mp4_list.push(path);
                }
                Ok(path) if path.contains(".jpg") => {
                    jpg_list.push(path);
                }
                _ => (),
            }
        }
    }

    // Print File lists
    println!("Downloaded file lists: ");
    for name in &mp4_list {
        println!("{}", name);
    }

    // Weak Slugify
    let revid_weak: Vec<String> = mp4_list
        .clone()
        .into_iter()
        .map(|x| weak_slugify(x))
        .collect();
    let remp3_weak: Vec<String> = revid_weak
        .clone()
        .into_iter()
        .map(|x| x.replace(".mp4", ".mp3"))
        .collect();

    // Rename mp4 file
    for (old, new) in mp4_list.into_iter().zip(&revid_weak) {
        let old_path = format!("temp_mp4_dir/{}", old);
        let new_path = format!("temp_mp4_dir/{}", new);
        rename(old_path, new_path)?;
    }

    println!("Rename file completed!");

    // Strong slugify
    let revid: Vec<String> = revid_weak
        .clone()
        .into_iter()
        .map(|x| strong_slugify(x))
        .collect();
    let remp3: Vec<String> = remp3_weak.into_iter().map(|x| strong_slugify(x)).collect();

    // Parallel convert
    revid
        .par_iter()
        .zip(remp3.par_iter())
        .for_each(|(vid, mp3)| {
            let vid_path = format!("temp_mp4_dir/{}", vid);
            let mp3_path = format!("mp3_dir/{}", mp3);
            let mut cmd = convert_mp3(&vid_path, &mp3_path);
            cmd.output().expect("Can't convert mp4 to mp3");
        });

    println!("Convert to mp3 completed!");

    // Remove mp4 files
    for vid in revid_weak {
        remove_file(format!("temp_mp4_dir/{}", vid))?;
    }

    println!("Remove mp4 files completed!");

    // Move jpg files
    for jpg in jpg_list {
        let old_path = format!("temp_mp4_dir/{}", jpg);
        let new_path = format!("mp3_dir/{}", jpg);
        rename(old_path, new_path)?;
    }

    println!("Move jpg files completed!");

    Ok(())
}

fn youtube_default() -> Command {
    let mut cmd = Command::new("youtube-dl");
    cmd.arg("-o")
        .arg("temp_mp4_dir/%(title)s.%(ext)s")
        .arg("--embed-thumbnail")
        .arg("--format")
        .arg("mp4");
    cmd
}

fn convert_mp3(name: &str, target: &str) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i")
        .arg(name)
        .arg("-b:a")
        .arg("320K")
        .arg("-vn")
        .arg(target);
    cmd
}

fn weak_slugify(s: String) -> String {
    s.replace(" ", "")
        .replace("./", "")
        .replace("&", "_")
}

fn strong_slugify(s: String) -> String {
//    s.replace("(", "\(").replace(")", "\)")
    s
}
