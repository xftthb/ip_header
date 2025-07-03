use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::Packet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;

fn main() {
    // 获取本地的网络接口
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| {
            iface.is_up() && !iface.is_loopback() && iface.ips.iter().any(|ip| ip.is_ipv4())
        })
        .expect("未找到可用的网络接口");

    println!("监听接口: {}", interface.name);

    // 创建数据通道
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("未实现的通道类型"),
        Err(e) => panic!("创建通道失败: {}", e),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                let ethernet = EthernetPacket::new(packet).unwrap();

                // 只处理 IPv4 数据包
                if ethernet.get_ethertype() == EtherTypes::Ipv4 {
                    if let Some(ip_packet) = Ipv4Packet::new(ethernet.payload()) {
                        // 只处理 UDP 包
                        if ip_packet.get_next_level_protocol() == IpNextHeaderProtocols::Udp {
                            let header_len = ip_packet.get_header_length() as usize * 4;
                            let base_header_len = 20;
                            println!("\nIP包:");
                            println!("  来源 IP:  {}", ip_packet.get_source());
                            println!("  目标 IP:  {}", ip_packet.get_destination());
                            println!("  协议号:   {}", ip_packet.get_next_level_protocol());
                            // 打印 Options 字段
                            if header_len > base_header_len {
                                let full_header = &ip_packet.packet()[..header_len];
                                let options = &full_header[base_header_len..];
                                print!("IP Options: ");
                                for b in options {
                                    print!("{:02x} ", b);
                                }
                                println!();
                            } else {
                                println!("无 IP Options 字段");
                            }
                            // 输出 identification 字段的值（即时间戳）
                            println!("  时间戳:   {}", ip_packet.get_identification());
                            if let Some(udp_packet) = UdpPacket::new(ip_packet.payload()) {
                                println!("UDP包:");
                                println!("  来源端口: {}", udp_packet.get_source());
                                println!("  目标端口: {}", udp_packet.get_destination());
                                println!("  长度:      {}", udp_packet.get_length());
                                println!("  数据内容:  {:?}", udp_packet.payload());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("接收包失败: {}", e);
            }
        }
    }
}
