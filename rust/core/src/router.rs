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

        let destination = cols[1];
        if destination != "00000000" {
            continue;
        }

        let gateway_hex = cols[2];
        let flags_hex = cols[3];
        let flags = u16::from_str_radix(flags_hex, 16).ok()?;
        if flags & 0x2 == 0 {
            continue;
        }

        let gw = u32::from_str_radix(gateway_hex, 16).ok()?;
        let b1 = (gw & 0xFF) as u8;
        let b2 = ((gw >> 8) & 0xFF) as u8;
        let b3 = ((gw >> 16) & 0xFF) as u8;
        let b4 = ((gw >> 24) & 0xFF) as u8;
        return Some(Ipv4Addr::new(b1, b2, b3, b4));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_proc_net_route_gateway;

    #[test]
    fn parse_gateway_from_proc_route() {
        let input = "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\neth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n";
        let ip = parse_proc_net_route_gateway(input).expect("gateway");
        assert_eq!(ip.to_string(), "192.168.1.1");
    }
}
