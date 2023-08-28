use std::net::IpAddr;

pub fn is_valid_server_address(input: &str) -> bool {
    // 尝试解析为 IP 地址
    if let Ok(ip_addr) = input.parse::<IpAddr>() {
        match ip_addr {
            IpAddr::V4(_) => {
                // 如果是 IPv4 地址，检查是否为合法地址
                true
            }
            IpAddr::V6(_) => {
                // 如果是 IPv6 地址，检查是否为全球单播地址
                true
            }
        }
    } else {
        false
    }
}

pub fn is_port(input: &str) -> bool {
    input.parse::<usize>().is_ok()
}
