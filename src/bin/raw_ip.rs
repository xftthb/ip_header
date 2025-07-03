use libc::{self, AF_INET, IPPROTO_RAW, SOCK_RAW, c_void, sockaddr, sockaddr_in};
use std::mem;
use std::net::Ipv4Addr;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

// IPv4头部结构 (符合RFC 791)
#[repr(C, packed)]
struct IPv4Header {
    version_ihl: u8, // 版本(4位) + 头部长度(4位)
    tos: u8,         // 服务类型
    total_len: u16,  // 总长度
    id: u16,         // 标识
    frag_off: u16,   // 分片偏移
    ttl: u8,         // 生存时间
    protocol: u8,    // 协议
    check: u16,      // 校验和
    saddr: u32,      // 源地址
    daddr: u32,      // 目的地址
                     // 选项字段(如果有)
}

// IP选项结构
/*struct IpOption {
    t: u8,         // Type
    length: u8,    // 选项总长度(包括类型和长度字段)
    data: Vec<u8>, // 选项数据
}*/

// UDP 头部结构 (符合 RFC 768)
#[repr(C, packed)]
struct UDPHeader {
    src_port: u16, // 源端口
    dst_port: u16, // 目的端口
    len: u16,      // UDP 数据报长度（头部 + 数据）
    check: u16,    // 校验和（可选，0 表示不计算）
}

// 计算校验和(适用于IP、ICMP等头部)
fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;

    // 处理16位字
    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            ((chunk[0] as u32) << 8) | (chunk[1] as u32)
        } else {
            (chunk[0] as u32) << 8
        };
        sum = sum.wrapping_add(word);
    }

    // 折叠进位
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16
}

// 构建IP选项字段
/*fn build_ip_options(options: IpOption) -> Vec<u8> {
    let mut result = Vec::new();

    result.push(options.t);
    result.push(options.length);
    result.extend_from_slice(&options.data);

    // 确保选项长度是4字节的倍数
    while result.len() % 4 != 0 {
        result.push(0); // 添加NOP或END选项填充
    }

    result
}*/

// 计算 UDP 伪头部校验和（包含 IP 源/目的地址）
fn udp_checksum(ip_header: &IPv4Header, udp_header: &UDPHeader, payload: &[u8]) -> u16 {
    let mut pseudo_header = Vec::new();

    // 伪头部（RFC 768）
    pseudo_header.extend_from_slice(&ip_header.saddr.to_be_bytes());
    pseudo_header.extend_from_slice(&ip_header.daddr.to_be_bytes());
    pseudo_header.push(0); // 保留
    pseudo_header.push(ip_header.protocol);
    pseudo_header.extend_from_slice(&udp_header.len.to_be_bytes());

    // UDP 头部 + 数据
    let mut udp_data = Vec::new();
    unsafe {
        udp_data.extend_from_slice(std::slice::from_raw_parts(
            udp_header as *const _ as *const u8,
            mem::size_of::<UDPHeader>(),
        ));
    }
    udp_data.extend_from_slice(payload);

    // 计算校验和
    let mut checksum_data = pseudo_header;
    checksum_data.extend(udp_data);
    checksum(&checksum_data)
}

fn main() {
    // 创建原始套接字
    let socket = unsafe { libc::socket(AF_INET, SOCK_RAW, IPPROTO_RAW) };
    if socket < 0 {
        eprintln!("无法创建原始套接字: {}", std::io::Error::last_os_error());
        process::exit(1);
    }

    // 设置IP_HDRINCL选项，告诉内核不要自动添加IP头部
    let on: i32 = 1;
    if unsafe {
        libc::setsockopt(
            socket,
            libc::IPPROTO_IP,
            libc::IP_HDRINCL,
            &on as *const _ as *const c_void,
            mem::size_of_val(&on) as libc::socklen_t,
        )
    } < 0
    {
        eprintln!(
            "无法设置IP_HDRINCL选项: {}",
            std::io::Error::last_os_error()
        );
        process::exit(1);
    }

    // 目标地址
    let dest_addr = Ipv4Addr::new(8, 8, 8, 8); // 例如Google DNS
    let dest_addr_u32 = u32::from_be_bytes(dest_addr.octets()); // 转换为网络字节序

    // 构建IP选项
/*     let options = IpOption {
        t: 0x79,
        length: 0x06,
        data: vec![0x12, 0x34, 0x56, 0x78],
    };

    let options_data = build_ip_options(options);
    let options_len = options_data.len();
    let ihl = 5 + (options_len / 4); // 计算IHL(32位字长度)
*/
    let ihl: i32 = 5;
    // 构建IP头部
    let mut ip_header = IPv4Header {
        version_ihl: (4 << 4) | (ihl as u8), // 版本4 + IHL
        tos: 0,
        total_len: 0, // 稍后填充
        id: 0,//稍后设置为时间戳
        frag_off: (0b010u16 << 13).to_be(), // Don't fragment标志
        ttl: 64,
        protocol: libc::IPPROTO_ICMP as u8, // 示例使用ICMP协议
        check: 0,                           // 稍后计算
        saddr: u32::from_be_bytes([192, 168, 1, 1]), // 示例源地址(网络字节序)
        daddr: dest_addr_u32,
    };

    // 构建负载数据
    let payload = b"Hello, raw IP packet with id!";

    // 计算总长度
    //let total_len = mem::size_of::<IPv4Header>() + options_len + payload.len();
    let total_len = mem::size_of::<IPv4Header>() + payload.len();
    ip_header.total_len = (total_len as u16).to_be();

    // 获取当前时间戳并设置为标识字段
    let timestamp = SystemTime::now()
       .duration_since(UNIX_EPOCH)
       .expect("时间戳获取失败")
       .as_secs() as u16;
    ip_header.id = timestamp.to_be();

    // 构建完整数据包
    let mut packet = Vec::with_capacity(total_len);
    unsafe {
        packet.extend_from_slice(std::slice::from_raw_parts(
            &ip_header as *const _ as *const u8,
            mem::size_of::<IPv4Header>(),
        ));
    }
    //packet.extend_from_slice(&options_data);
    packet.extend_from_slice(payload);

    // 计算校验和(跳过校验和字段)
    let check = checksum(&packet[0..mem::size_of::<IPv4Header>()]);
    packet[10] = (check >> 8) as u8;
    packet[11] = check as u8;

    // 目标地址结构
    let mut dest_sockaddr = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: 0, // 端口对IP层不重要
        sin_addr: libc::in_addr {
            s_addr: dest_addr_u32,
        },
        sin_zero: [0; 8],
    };

    // 发送数据包
    let send_result = unsafe {
        libc::sendto(
            socket,
            packet.as_ptr() as *const c_void,
            packet.len(),
            0,
            &dest_sockaddr as *const _ as *const sockaddr,
            mem::size_of_val(&dest_sockaddr) as libc::socklen_t,
        )
    };

    if send_result < 0 {
        eprintln!("发送数据包失败: {}", std::io::Error::last_os_error());
    } else {
        println!("成功发送 {} 字节的数据包", send_result);
    }

    // 关闭套接字
    unsafe { libc::close(socket) };
}
