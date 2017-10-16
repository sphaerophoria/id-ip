use ::errors::*;
use std::str::FromStr;
use std::env;
use hwaddr::HwAddr;

#[allow(dead_code)]
pub fn format_mac(mac: &HwAddr) -> String {
    let octets = mac.octets();
    let mut output_str = format!("{:02x}", octets[0]);
    for octet in octets.iter().skip(1) {
        output_str = format!("{}:{:02x}", output_str, octet);
    }

    output_str
}

pub fn get_mac(id: &str) -> Result<HwAddr> {
    let id_key = format!("{}_mac", id);

    let mac_str = env::var(id_key)
        .chain_err(|| "Cannot find mac for id")?;

    let mac = HwAddr::from_str(&mac_str)
        .chain_err(|| "Failed to parse mac for id")?;

    Ok(mac)
}
