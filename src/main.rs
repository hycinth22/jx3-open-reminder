use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use getopts::Occur;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;
use args::{Args, ArgsError};
use tokio::select;
use log::{error, info, trace};
use rodio::Source;

struct ServerInfo {
    server: String,
    host: String,
    port: u16,
}


#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    // load music
    let (audio_output_stream, audio_stream_handle) = rodio::OutputStream::try_default().unwrap();
    let open_audio_source = {
        let file = File::open("open.flac").unwrap();
        rodio::Decoder::new(BufReader::new(file)).unwrap()
    };
    let mut open_audio_source_samples = Option::Some(open_audio_source.convert_samples());

    // Create the runtime
    match parse(&std::env::args().collect()) {
        Ok((monitored_servers, interval)) => {
            let servers_info = download_server_list().await.expect("download_server_list failed");
            for (_, server_info) in &servers_info {
                print!("\"{}\" ", server_info.server);
            }
            println!("以上是所有的服务器名称。");


            for server in monitored_servers {
                if !servers_info.contains_key(&server) {
                    error!("服务器 {} 不存在", server);
                    exit(1);
                }
                select! {
                     _ = monitor_server(&server, &servers_info[&server].host, servers_info[&server].port, interval) => {
                        info!("{server} 开服了！");
                        let _ignore_error = audio_stream_handle.play_raw(open_audio_source_samples.take().unwrap());
                        alert::alert("开服了", &format!("{server} 开服了！"));
                    }
                    else => break,
                }
            }
        },
        Err(error) => {
            error!("{}", error);
            exit(1);
        }
    };
}

fn parse(input: &Vec<String>) -> Result<(Vec<String>, u64), ArgsError> {
    const PROGRAM_DESC: &'static str = "This is a monitor of JX3 game server opening";
    const PROGRAM_NAME: &'static str = "JX3 Open Monitor";
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print the usage menu");
    args.option("s",
                "server",
                "The server to monitor",
                "SERVER",
                Occur::Req,
                Some("天鹅坪".to_string()));
    args.option("i",
                "interval",
                "The interval to detect",
                "INTERVAL",
                Occur::Optional,
                Some(100.to_string()));
    args.parse(input)?;

    let help = args.value_of("help")?;
    if help {
        args.full_usage();
        exit(0);
    }

    let servers = args.values_of("server")?;
    println!("监控的服务器列表：{:?}", servers);
    let interval: u64 = args.value_of("interval")?;
    println!("探测间隔 {:?}", interval);
    Ok((servers, interval))
}

async fn download_server_list() -> Result<HashMap<String, ServerInfo>, reqwest::Error> {
    use encoding::all::GBK;
    use encoding::{DecoderTrap, Encoding};
    let body_gbk_bytes = reqwest::get("http://jx3comm.xoyocdn.com/jx3hd/zhcn_hd/serverlist/serverlist.ini")
        .await?
        .bytes()
        .await?;

    let body = GBK.decode(&*body_gbk_bytes, DecoderTrap::Strict).unwrap();

    Ok(body.split('\n').map(|line| {
        let mut elems = line.split('\t');
        let server = elems.nth(1).unwrap().to_string();
        (server.clone(), ServerInfo {
            server,
            host: elems.nth(1).unwrap().to_string(),
            port: elems.nth(0).unwrap().parse::<u16>().unwrap(),
        })
    }).collect())
}


async fn monitor_server(server: &str, host: &str, port: u16, interval: u64) {
    use tokio::net::TcpSocket;
    let retry_duration = Duration::from_millis(interval);
    info!("正在监控 {}", server);
    let addr = format!("{server}:{port}");
    let host = Ipv4Addr::from_str(host).expect("解析ip地址错误");
    let addr = SocketAddr::new(IpAddr::from(host), port);
    loop {
        let socket = TcpSocket::new_v4().unwrap();
        // Connect
        let stream = socket.connect(addr).await;
        if stream.is_ok() {
            // info!("{server} ({addr}) connect successfully");
            return
        } else {
            trace!("{retry_duration:?}后重新探测 {server}" );
            tokio::time::sleep(retry_duration).await
        }
    }
}
