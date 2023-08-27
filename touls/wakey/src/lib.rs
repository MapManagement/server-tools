use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, UdpSocket};
use std::num::ParseIntError;

pub fn wake_on_lan(mac_address: &str) -> Result<(), Error> {
    {
        let parse_result = mac_to_array(mac_address);

        match parse_result {
            Ok(address_array) => {
                let magic_packet_result = build_magic_packet(address_array);

                match magic_packet_result {
                    Ok(magic_packet) => send_packet(magic_packet),
                    Err(message) => return Err(Error::new(ErrorKind::Other, message)),
                }
            }
            Err(message) => return Err(Error::new(ErrorKind::InvalidData, message)),
        };
    }

    Ok(())
}

fn mac_to_array(mac_address: &str) -> Result<[u8; 6], String> {
    let address_bytes: Vec<&str> = mac_address.split(":").collect();
    let mut address_array: [u8; 6] = [0; 6];

    for i in 0..address_array.len() {
        let parsed = decode_hex(address_bytes[i]);

        match parsed {
            Ok(address_int) => address_array[i] = address_int,
            Err(_) => return Err("The given MAC address is not valid".to_string()),
        };
    }

    Ok(address_array)
}

fn decode_hex(hex_string: &str) -> Result<u8, ParseIntError> {
    return u8::from_str_radix(&hex_string[0..2], 16);
}

fn build_magic_packet(address_array: [u8; 6]) -> Result<[u8; 102], String> {
    // broadcast   - 6 *  1
    // mac address - 6 * 16
    // total        - 102
    let mut muliplied_array: [u8; 102] = [0; 102];

    for i in 0..6 {
        muliplied_array[i] = 0xff;
    }
    let mut counter = 6;

    for _ in 1..=16 {
        for j in 0..address_array.len() {
            muliplied_array[counter] = address_array[j];
            counter += 1;
        }
    }

    Ok(muliplied_array)
}

fn send_packet(magic_packet: [u8; 102]) {
    println!("{:?}", magic_packet);
    let socket =
        UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to address!");
    socket
        .set_broadcast(true)
        .expect("Cannot send to broadcast!");
    let _repsonse = socket.send_to(&magic_packet, "255.255.255.255:9");
}
