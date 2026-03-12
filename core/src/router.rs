use std::net::Ipv4Addr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("default gateway not found")]
    NotFound,
    #[error("read route table failed: {0}")]
    ReadFailed(#[from] std::io::Error),
}

pub fn get_gateway_v4() -> Result<Ipv4Addr, GatewayError> {
    #[cfg(target_os = "linux")]
    {
        let content = std::fs::read_to_string("/proc/net/route")?;
        if let Some(ip) = parse_proc_net_route_gateway(&content) {
            return Ok(ip);
        }
        return Err(GatewayError::NotFound);
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(GatewayError::NotFound)
    }
}

fn parse_proc_net_route_gateway(content: &str) -> Option<Ipv4Addr> {
    for line in content.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        let dest = match cols.get(1) {
            Some(v) => *v,
            None => continue,
        };
        if dest != "00000000" {
            continue;
        }
        let flags_hex = match cols.get(3) {
            Some(v) => *v,
            None => continue,
        };
        let flags = u16::from_str_radix(flags_hex, 16).ok()?;
        if flags & 0x2 == 0 {
            continue;
        }
        let gw_hex = match cols.get(2) {
            Some(v) => *v,
            None => continue,
        };
        let gw = u32::from_str_radix(gw_hex, 16).ok()?;
        let b1 = (gw & 0xFF) as u8;
        let b2 = ((gw >> 8) & 0xFF) as u8;
        let b3 = ((gw >> 16) & 0xFF) as u8;
        let b4 = ((gw >> 24) & 0xFF) as u8;
        return Some(Ipv4Addr::new(b1, b2, b3, b4));
    }
    None
}
