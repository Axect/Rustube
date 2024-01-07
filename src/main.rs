extern crate rayon;
use rayon::prelude::*;

use std::fs::{remove_file, rename, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let link = File::open("link.txt")?;
    let reader = BufReader::new(link);

    // Youtube download
    // print stdout, stderr
    for line in reader.lines() {
        let line = line?;
        println!("Downloading {}...", line);
        //let mut cmd = youtube_default();
        let mut cmd = youtube_to_flac();
        cmd.arg(line);
        let stdout = cmd.output().expect("Can't download");
        println!("{}", String::from_utf8_lossy(&stdout.stdout));
        println!("{}", String::from_utf8_lossy(&stdout.stderr));
    }

    println!("Download completed!");

    let mut mp4_list: Vec<String> = Vec::new();
    let mut webp_list: Vec<String> = Vec::new();

    // // Find mp4 files
    // for file in std::fs::read_dir("temp_mp4_dir")? {
    //     let entry = file?;
    //     let path = entry.path();
    //     if !path.is_dir() {
    //         match entry.file_name().into_string() {
    //             Ok(path) if path.contains(".mp4") => {
    //                 mp4_list.push(path);
    //             }
    //             Ok(path) if path.contains(".webp") => {
    //                 webp_list.push(path);
    //             }
    //             _ => (),
    //         }
    //     }
    // }
    // Find flac files
    for file in std::fs::read_dir("temp_mp4_dir")? {
        let entry = file?;
        let path = entry.path();
        if !path.is_dir() {
            match entry.file_name().into_string() {
                Ok(path) if path.contains(".flac") => {
                    mp4_list.push(path);
                }
                Ok(path) if path.contains(".webp") => {
                    webp_list.push(path);
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
    //let remp3_weak: Vec<String> = revid_weak
    //    .clone()
    //    .into_iter()
    //    .map(|x| x.replace(".mp4", ".mp3"))
    //    .collect();

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
    //let remp3: Vec<String> = remp3_weak.into_iter().map(|x| strong_slugify(x)).collect();

    // Parallel convert
    revid
        .par_iter()
        //.zip(remp3.par_iter())
        .for_each(|vid| {
            let vid_path = format!("temp_mp4_dir/{}", vid);
            let mp3_path = format!("mp3_dir/{}", vid);
            rename(vid_path, mp3_path).unwrap();
            //let mut cmd = convert_mp3(&vid_path, &mp3_path);
            //cmd.output().expect("Can't convert mp4 to mp3");
        });

    println!("Convert to mp3 completed!");

    // Move webp files
    let mut webp_weak_list: Vec<String> = Vec::new();
    for webp in webp_list.iter() {
        let old_path = format!("temp_mp4_dir/{}", webp);
        let webp_weak = weak_slugify(webp.clone());
        let new_path = format!("mp3_dir/{}", webp_weak);
        rename(old_path, new_path)?;
        webp_weak_list.push(webp_weak);
    }

    // Convert webp to png using dwebp
    for webp in webp_weak_list.iter() {
        let webp_path = format!("mp3_dir/{}", webp);
        let mut cmd = Command::new("dwebp");
        cmd.arg(webp_path.clone()).arg("-o").arg(webp_path.replace(".webp", ".png"));
        cmd.output().expect("Can't convert webp to png");
    }

    println!("Convert webp to png completed!");

    // Remove webp files
    for webp in webp_weak_list.iter() {
        remove_file(format!("mp3_dir/{}", webp))?;
    }

    println!("Remove webp files completed!");

    //// Remove mp4 files
    //for vid in revid_weak {
    //    remove_file(format!("temp_mp4_dir/{}", vid))?;
    //}
    //
    //println!("Remove mp4 files completed!");

    Ok(())
}

fn youtube_default() -> Command {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-o")
        .arg("temp_mp4_dir/%(title)s.%(ext)s")
        .arg("--write-thumbnail")
        .arg("--format")
        .arg("mp4");
    cmd
}

fn youtube_to_flac() -> Command {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-o")
        .arg("temp_mp4_dir/%(title)s.%(ext)s")
        .arg("--write-thumbnail")
        .arg("-x")
        .arg("--audio-format")
        .arg("flac")
        .arg("--audio-quality")
        .arg("0")
        .arg("--embed-metadata");
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
