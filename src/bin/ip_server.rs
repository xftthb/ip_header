use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4};
use std::os::unix::prelude::*;
use std::io::{self, Read};
use libc;

fn main() -> io::Result<()> {
    // 创建原始套接字，协议为IPPROTO_UDP
    let socket = unsafe {
        std::ffi::CString::new("raw")
            .unwrap()
            .as_ptr()
    };
    let sockfd = unsafe {
        libc::socket(
            libc::AF_INET,
            libc::SOCK_RAW,
            libc::IPPROTO_UDP,
        )
    };
    if sockfd < 0 {
        return Err(io::Error::last_os_error());
    }

    println!("开始监听 IP 数据报...");

    let mut buf = [0u8; 65535]; // 最大IP包的大小

    loop {
        let mut len = buf.len() as libc::socklen_t;
        let mut src: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let amt = unsafe {
            libc::recvfrom(
                sockfd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
                &mut src as *mut _ as *mut _,
                &mut len as *mut _,
            )
        };

        if amt < 0 {
            return Err(io::Error::last_os_error());
        }

        let amt = amt as usize;

        // 从 sockaddr_in 中提取 IP 和端口
        let ip = Ipv4Addr::new(
            (src.sin_addr.s_addr >> 24) as u8,
            (src.sin_addr.s_addr >> 16) as u8,
            (src.sin_addr.s_addr >> 8) as u8,
            src.sin_addr.s_addr as u8,
        );
        let port = src.sin_port.to_be() as u16;
        let src_addr = SocketAddrV4::new(ip, port);

        // IP包头的最小长度为20字节
        if amt < 20 {
            println!("收到来自 {}: 数据包过短，无法解析IP包头", src_addr);
            continue;
        }

        // 提取IP包头（前20字节）
        let ip_header = &buf[..20];

        // 提取IPv4 Identification字段（第4-5字节，即索引3和4）
        let identification = &ip_header[3..5];

        // 转换为16进制字符串
        let hex_id = format!("{:02x}{:02x}", identification[0], identification[1]);
        
        println!("收到来自 {}: Identification字段=0x{}", src_addr, hex_id);
        
        // 其他数据处理逻辑保持不变...
        println!("数据长度: {} 字节", amt);

        // 打印十六进制数据
        print!("IP包头(hex): ");
        for b in &ip_header[..20] {  // 仅打印IP包头
            print!("{:02x} ", b);
        }
        println!();

        // UDP数据在IP包头之后
        let udp_start = 20;
        
        // UDP数据包的最小长度为8字节（UDP头）
        if amt < udp_start + 8 {
            println!("数据包过短，无法解析UDP头");
            continue;
        }

        // UDP头的长度为8字节，负载在UDP头之后
        let udp_payload_start = udp_start + 8;
        
        // 确保不会越界
        let udp_payload_len = amt.saturating_sub(udp_payload_start);
        
        // 打印UDP负载
        if udp_payload_len > 0 {
            let udp_payload = &buf[udp_payload_start..udp_payload_start + udp_payload_len];
            
            print!("UDP负载(hex): ");
            for b in udp_payload {
                print!("{:02x} ", b);
            }
            println!();
            
            // 尝试打印为字符串，只针对UDP负载
            match std::str::from_utf8(udp_payload) {
                Ok(s) => println!("UDP负载字符串内容: {}", s),
                Err(_) => println!("UDP负载（非 UTF-8 数据）"),
            }
        } else {
            println!("UDP数据包没有负载");
        }
    }
}