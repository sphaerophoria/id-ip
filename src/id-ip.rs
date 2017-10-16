#[macro_use] extern crate error_chain;
extern crate linear_map;
extern crate oping;
extern crate tempfile;
extern crate getifaddrs;
extern crate hwaddr;

mod utils;
mod errors;

use errors::*;

use linear_map::LinearMap;
use std::net::Ipv4Addr;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::str::FromStr;
use std::fs;
use oping::Ping;
use hwaddr::HwAddr;


quick_main!(run);

// This function is terrrrribly inefficient
fn find_in_arp_table(mac: &HwAddr) -> Result<Ipv4Addr> {
    let f = File::open("/proc/net/arp").chain_err(||"Failed to open arp file")?;
    let reader = BufReader::new(f);
    let v = reader
        .lines()
        .skip(1)
        .map(|s| s.unwrap().split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>())
        .map(|v| ( HwAddr::from_str(&v[3]).unwrap(), (Ipv4Addr::from_str(&v[0]).unwrap())))
        .collect::<LinearMap<_,_>>();

    Ok(v.get(&mac).ok_or("Failed to find mac in arp table")?.clone())
}


fn ping_all_on_subnets() {
    for addr in &getifaddrs::get_if_addrs().unwrap() {
        let mut ping = Ping::new();
        ping.set_timeout(0.5).unwrap();

        let netmask: u32 = unsafe {std::mem::transmute(addr.netmask.octets())};

        let mut netmask_positions = vec![];
        for i in 0..32 {
            if (netmask >> i) & 1 == 0 {
                netmask_positions.push(i);
            }
        }

        if netmask_positions.len() > 9 {
            continue;
        }

        let mut current_addr: u32 = unsafe {std::mem::transmute(addr.addr.octets())};
        current_addr &= netmask;

        for i in 1..(1 << netmask_positions.len()) - 1 {
            for bit_pos in 0..netmask_positions.len() {
                current_addr &= !(1<< netmask_positions[bit_pos]);
                current_addr |= (1<<bit_pos & i) << netmask_positions[bit_pos] - bit_pos;
            }
            let addr = unsafe{ std::mem::transmute::<_, [u8; 4]>(current_addr)};
            let host = format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]);
            println!("host: {}", host);
            ping.add_host(&host).expect("Cannot add host");
        }

        // Execute ping
        let _ = ping.send();
    }
}

fn run() -> Result<()> {
    let id = std::env::args().nth(1).ok_or("No provided mac address")?;
    let mac = utils::get_mac(&id)?;
    let mut ip = find_in_arp_table(&mac);
    if ip.is_err() {
        // do ping
        ping_all_on_subnets();
        ip = find_in_arp_table(&mac);
    }

    let ip = ip?;
    println!("{}", ip);

    let hosts_file_in  = BufReader::new(File::open("/etc/hosts").chain_err(||"Failed to open hosts file")?);
    let mut hosts_file_out = tempfile::NamedTempFile::new().chain_err(||"Failed to create temp file")?;
    let perms = fs::metadata("/etc/hosts").chain_err(||"failed to get hosts metadata")?.permissions();
    fs::set_permissions(hosts_file_out.path(), perms).chain_err(||"Failed to set permissions")?;
    let mut host_written = false;

    'outer: for line in hosts_file_in.lines() {
        let line = line.chain_err(||"Failed to parse hosts line")?;
        let line_elems: Vec<_> = line.split_whitespace().collect();

        for elem in line_elems.iter().skip(1) {
            if *elem == id {
                if Ipv4Addr::from_str(line_elems[0]).chain_err(||"Failed to parse ip")? == ip {
                    return Ok(());
                }
                write!(hosts_file_out, "{} {}\n", ip, id).chain_err(||"Failed to write hosts file")?;
                host_written = true;
                continue 'outer;
            }
        }

        write!(hosts_file_out, "{}\n", line).chain_err(||"Failed to write hosts file")?;
    }

    if !host_written {
        write!(hosts_file_out, "{} {}\n", ip, id).chain_err(|| "Failed to write hosts file")?;
    }

    std::fs::copy(hosts_file_out.path(), "/etc/hosts").chain_err(||"Failed to copy over hosts file")?;

    Ok(())
}