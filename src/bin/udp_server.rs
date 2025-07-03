use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8001")?;
    println!("服务端监听 UDP 端口 8001...");

    let mut buf = [0u8; 1500]; // 最大 UDP 包的大小
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        
        // IPv4包的最小长度为20字节（包头）
        if amt < 20 {
            println!("收到来自 {}: 数据包过短，无法解析IPv4包头", src);
            continue;
        }

        // 提取IPv4包头（前20字节）
        let ip_header = &buf[..20];

        // 提取IPv4 Identification字段（第4-5字节，即索引3和4）
        let identification = &ip_header[3..5];

        // 转换为16进制字符串
        let hex_id = format!("{:02x}{:02x}", identification[0], identification[1]);
        
        println!("收到来自 {}: Identification字段=0x{}", src, hex_id);
        
        // 其他数据处理逻辑保持不变...
        println!("数据长度: {} 字节", amt);

        // 打印十六进制数据
        print!("数据内容(hex): ");
        for b in &ip_header[..20] {  // 仅打印IP包头
            print!("{:02x} ", b);
        }
        println!();

        // 打印完整数据
        print!("完整数据(hex): ");
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