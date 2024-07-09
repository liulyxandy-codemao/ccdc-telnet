use std::fs;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const TELNET_HEAD: [u8; 12] = [
    0xFF, 0xFB, 0x01, 0xFF, 0xFD, 0x01, 0xFF, 0xFB, 0x03, 0xFF, 0xFC, 0x1F,
];

#[allow(dead_code)]
fn translate(buf: &[u8]) {
    print!("\nTranslate\n");
    for &i in buf {
        match i {
            255 => print!("\nIAC (字符0xFF)"), // IAC
            254 => print!("DONT (选项协商)"),  // DONT
            253 => print!("DO (选项协商)"),    // DO
            252 => print!("WONT (选项协商)"),  // WONT
            251 => print!("WILL (选项协商)"),  // WILL
            250 => print!("SB (子选项开始)"),  // SB
            249 => print!("GA (继续)"),        // GA
            248 => print!("EL (擦除一行)"),    // EL
            247 => print!("EC (擦除字符)"),    // EC
            246 => print!("AYT (你在吗?)"),    // AYT
            245 => print!("AO (终止输出)"),    // AO
            244 => print!("IP (中断进程)"),    // IP
            243 => print!("BRK (断开)"),       // BRK
            242 => print!("DM (数据标记)"),    // DM
            241 => print!("NOP (无操作)"),     // NOP
            240 => print!("SE (子选项结束)"),  // SE
            239 => print!("EOR (记录结束)"),   // EOR
            238 => print!("ABORT (中止)"),     // ABORT
            237 => print!("SUSP (暂停)"),      // SUSP
            236 => print!("EOF (文件结束)"),   // EOF
            1 => print!("回应(回显)"),         // 回应
            3 => print!("禁止继续"),           // 禁止继续
            5 => print!("状态"),               // 状态
            6 => print!("时钟标识"),           // 时钟标识
            24 => print!("终端类型"),          // 终端类型
            31 => print!("窗口大小"),          // 窗口大小
            32 => print!("终端速率"),          // 终端速率
            33 => print!("远端流量控制"),      // 远端流量控制
            34 => print!("行模式"),            // 行模式
            36 => print!("环境变量"),          // 环境变量
            _ => print!("{}", i as char),
        }
    }
}


async fn welcome_message(stream: &mut TcpStream){
    let welcome_message = b"\r
Welcome to the verify server!\r
You should enter your code after the determiner '~>'.\r
You should NOT input characters other than letters and numbers.\r
If you do, your input will be rejected.\r
Due to some reasons, you could not edit your input.\r
Press 'Ctrl+C' to quit.\r
";
    let _ = stream.write(welcome_message).await;
}

async fn determiner(stream: &mut TcpStream){
    let determiner = b"\r\n~> ";
    let _ = stream.write(determiner).await;
}

async fn check(answer: String, stream: &mut TcpStream, addr: SocketAddr) -> Result<(), ()> {
    if answer == String::from("8AE4F85617F021F5C15CF029656E063BB700E7A3") {
        let random_number = rand::random::<u32>();
        fs::write("codes.txt", 
            format!("{}\n{}", String::from_utf8(fs::read("codes.txt").unwrap()).unwrap(), random_number)
        ).unwrap();
        fs::write("addr.txt", 
            format!("{}\n{}", String::from_utf8(fs::read("addr.txt").unwrap()).unwrap(), addr.ip().to_string())
        ).unwrap();
        let success_message = format!("\r
Wow! You have passed the verification!\r
Please join our QQ group: 970907767\r
with your unique code: {}\r
to get your prize!\r
Connection will close now.\r",random_number);
        let _ = stream.write(success_message.as_bytes()).await;
        Ok(())
    } else {
        let fail_message = b"\r
Uh-oh! You have failed the verification!\r
Please try again.\r";
        let _ = stream.write(fail_message).await;
        Err(())
    }
}

async fn err_unknown(stream: &mut TcpStream) {
    let unknown_message = b"\r
Uh-oh! We met some problem during setting up!\r
Details:\r
 - It seems that you already finished the verification.\r
 - You can NOT enter the server again.\r";
    let _ = stream.write(unknown_message).await;
}

async fn handler(buf: &[u8], stream: &mut TcpStream) -> Result<String, bool> {
    let mut res = String::from("");

    for &i in buf {
        match i {
            255 => return Err(false),
            1 => return Err(false),  // 回应
            3 => return Err(true),   // 禁止继续
            5 => return Err(false),  // 状态
            6 => return Err(false),  // 时钟标识
            24 => return Err(false), // 终端类型
            31 => return Err(false), // 窗口大小
            32 => return Err(false), // 终端速率
            33 => return Err(false), // 远端流量控制
            34 => return Err(false), // 行模式
            36 => return Err(false), // 环境变量
            _ => {
                if !(i >= 65 && i <= 90) && !(i >= 97 && i <= 122) && !(i >= 48 && i <= 57) && !(i == 13) && !(i == 10) {
                    return Err(false);
                }
                res.insert(res.len(), i as char);
            }
        }
    }
    let _ = stream.write(buf).await;
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:4567").await?;
    println!("Server listening on port 4567");
    while let Ok(stream) = listener.accept().await {
        tokio::task::spawn(async move {
            let (mut stream, addr) = stream;
            stream.write_all(&TELNET_HEAD[..]).await.unwrap();
            let addrs = String::from_utf8(fs::read("addr.txt").unwrap()).unwrap();
            if addrs.contains(&addr.ip().to_string()) {
                err_unknown(&mut stream).await;
                return;
            }
            let mut buffer = [0; 1024];
            let mut ans = String::new();
            welcome_message(&mut stream).await;
            determiner(&mut stream).await;
            loop {
                let n = stream.read(&mut buffer).await.unwrap();
                let resp = handler(&buffer[..n], &mut stream).await;
                
                match resp {
                    Ok(res) => {
                        ans = if res == String::from("\n") || res == String::from("\r\n") || res == String::from("\r"){
                            match check(ans, &mut stream, addr).await {
                                Ok(()) => break,
                                Err(()) => {}
                            };
                            determiner(&mut stream).await;
                            String::new()
                        } else {
                            format!("{}{}", ans, res)
                        };
                    },
                    Err(true) => break,
                    Err(false) => continue,
                }
                if n == 0 {
                    break;
                }
            }
        });
    }
    Ok(())
}
