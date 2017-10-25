use ::errors::*;
use std::str::FromStr;
use std::env;
use eui48::MacAddress;

pub fn get_mac(id: &str) -> Result<MacAddress> {
    let id_key = format!("{}_mac", id);

    let mac_str = env::var(id_key)
        .chain_err(|| "Cannot find mac for id")?;

    let mac = MacAddress::from_str(&mac_str)
        .chain_err(|| format!("Failed to get mac address from {}", mac_str))?;

       // .map_err(|e| ErrorKind::ParseMacAddrErr(format!("{:?}", e)))?;

    Ok(mac)
}
