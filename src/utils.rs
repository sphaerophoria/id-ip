use ::errors::*;
use std::str::FromStr;
use std::env;
use pnet::util::MacAddr;

pub fn get_mac(id: &str) -> Result<MacAddr> {
    let id_key = format!("{}_mac", id);

    let mac_str = env::var(id_key)
        .chain_err(|| "Cannot find mac for id")?;

    let mac = MacAddr::from_str(&mac_str)
        .map_err(|e| ErrorKind::ParseMacAddrErr(format!("{:?}", e)))?;

    Ok(mac)
}
