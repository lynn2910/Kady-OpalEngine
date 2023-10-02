use std::io::Read;
use std::process::{Command, exit, Stdio};
use std::sync::Arc;
use std::time::Duration;
use clap::Parser;
use colored::Colorize;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Receiver;
use tokio::time::Instant;
use config::Config;

/// Create needed folders & files
fn pre_start(config: &Config){
    // check if the folder defined at config.logs_path exists
    if !std::path::Path::new(&config.logs_path).is_dir() {
        std::fs::create_dir(&config.logs_path).expect("Cannot create logs folder");
    }

    if !std::path::Path::new(&config.security.archive_path).exists() {
        println!("{}", "Cannot find archive folder, security is compromised and the core cannot be started".red().bold());
    }
}

async fn run_process(
    cli_args: &CliArgs,
    config: &Config,
    receive_kill_request: Arc<Mutex<Receiver<()>>>
) -> i32 {
    let mut command = Command::new(config.core_path.as_str());

    let command = command
        .arg("--port")
        .arg(cli_args.port.to_string())
        .arg("--domain")
        .arg(&cli_args.domain)
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit());

    let mut child_process = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            fatal_error(e.to_string().as_str(), 1);
            return 1;
        },
    };

    let mut stdout = child_process.stdin.take().expect("Cannot acquire stdout");

    let h = tokio::spawn(async move {
        let mut receive_kill_request = receive_kill_request.lock().await;
        if receive_kill_request.try_recv().is_ok() {
            match child_process.kill() {
                Ok(_) => child_process.wait().map(|s| s.code().unwrap_or(1000)).unwrap_or(2000),
                Err(e) => {
                    fatal_error(format!("Cannot kill the process after receiving a kill request: {e:#?}").as_str(), 1);
                    3000
                }
            }
        } else {
            child_process.wait().map(|s| s.code().unwrap_or(1000)).unwrap_or(2000)
        }
    });


    // finally, wait for the child to exit and get the exit code
    while stdout.as_handle(&mut [0; 1024]).is_ok() {

    }

    h.await.unwrap_or(4000)
}



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// The domain that the API will listen to
    #[arg(short, long)]
    domain: String,

    /// The port of which the API will be listening to
    #[arg(short, long)]
    port: u16
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    // load config
    let config: Config = config::load_from("config.toml".to_string()).expect("Cannot load config.toml");

    pre_start(&config);

    let mut last_crashes: Vec<Instant> = vec![];

    let (tx, rw) = tokio::sync::oneshot::channel::<()>();
    let receive_kill_request = Arc::new(Mutex::new(rw));

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            println!("{}", "Exiting...".green());
            let _ = tx.send(());
        };
    });

    loop {
        print!("{}", "Starting process...".green());

        let exit_code = run_process(
            &args,
            &config,
            receive_kill_request.clone()
        ).await;

        if exit_code == 0 {
            break;
        } else {
            print!("{} {}", "Process exited with code ".red(), exit_code.to_string().red().bold());

            last_crashes.push(Instant::now());
            last_crashes.retain(|i| i > &(Instant::now() - Duration::from_secs(120)));

            if last_crashes.len() >= 5 {
                return fatal_error(
                    "Too many crashes in the last minute, exiting.\nNo more than 5 crashes occured in the last 120 seconds",
                    1
                );
            }
        };
    }

    println!("{}", "Graceful exit, so you soon".green())
}

fn fatal_error<T: ToString + Colorize>(error: T, code: i32){
    println!("\n{}{}", "[ERROR] - ".red(), error.red().bold());

    exit(code);
}