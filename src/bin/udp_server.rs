use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    // 绑定本地地址和端口
    let socket = UdpSocket::bind("0.0.0.0:8001")?;
    println!("服务端监听 UDP 端口 8001...");

    let mut buf = [0u8; 1500]; // 最大 UDP 包的大小
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("收到来自 {} 的数据: {} 字节", src, amt);

        // 打印十六进制数据
        print!("数据内容(hex): ");
        for b in &buf[..amt] {
            print!("{:02x} ", b);
        }
        println!();

        // 尝试打印为字符串
        match std::str::from_utf8(&buf[..amt]) {
            Ok(s) => println!("字符串内容: {}", s),
            Err(_) => println!("（非 UTF-8 数据）"),
        }
    }
}
