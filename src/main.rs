use clap::Parser;
use std::error;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(name = "sneakin-commits")]
struct Args {
    #[arg(long, env = "SNEAKIN_COMMITS_MAGIC_WORD", default_value = "a")]
    magic_word: String,
    #[arg(long, env = "SNEAKIN_COMMITS_EMAIL")]
    mail: String,
    #[arg(long, env = "SNEAKIN_COMMITS_MAX_COUNT", default_value = "100")]
    max_count: i32,
}

#[derive(Debug)]
struct Commit {
    hash: String,
    author: String,
    message: String,
}

fn main() {
    let result = command();
    match result {
        Ok(_) => println!("done"),
        Err(e) => println!("error: {}", e),
    }
}

fn command() -> Result<(), Box<dyn error::Error>> {
    let check_output = Command::new("which").arg("git").output()?;
    println!("{:?}", check_output);
    match check_output.status.code() {
        Some(0) => (),
        _ => return Err("git is not installed".into()),
    }
    let args = Args::parse();
    if args.mail == "" {
        return Err("SNEAKIN_COMMITS_EMAIL is not set".into());
    }
    let output = Command::new("git")
        .arg("log")
        .arg(r#"--pretty=format:%H,%ae,%s"#)
        .arg(format!("-n{}", args.max_count))
        .output()?;
    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    let mut commits: Vec<Commit> = vec![];
    for line in String::from_utf8_lossy(&output.stdout).split("\n") {
        let commit = Commit {
            hash: line.split(",").collect::<Vec<&str>>()[0].to_string(),
            author: line.split(",").collect::<Vec<&str>>()[1].to_string(),
            message: line.split(",").collect::<Vec<&str>>()[2].to_string(),
        };
        commits.push(commit);
    }
    let mut target_hashes: Vec<String> = vec![];
    let mut target_messsage = "".to_string();
    for commit in commits {
        target_messsage = commit.message.clone();
        if commit.author != args.mail {
            break;
        }
        if commit.message == args.magic_word {
            println!("{:?}", commit);
            target_hashes.push(commit.hash);
        } else {
            break;
        }
    }
    if target_hashes.len() == 0 {
        return Err("no target commits found".into());
    }
    println!("target_hashes: {:?}", target_hashes);
    let mut target_commit: String = "HEAD".to_string();
    for _ in 0..target_hashes.len() + 1 {
        target_commit = format!("{}^", target_commit);
    }
    println!("target_commit: {}", target_commit);
    let _output = Command::new("git")
        .arg("reset")
        .arg("--soft")
        .arg(target_commit)
        .output()?;
    let _output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(format!("{}", target_messsage))
        .output()?;
    Ok(())
}
