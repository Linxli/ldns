use crate::{blocklookup, resolver};
use hickory_proto::{
    op::{Message, MessageType, OpCode, Query, ResponseCode},
    rr::{Name, RData, Record, RecordType, rdata},
    serialize::binary::{BinEncodable, BinEncoder},
};
use std::net::{IpAddr, Ipv4Addr};
use tokio::net::UdpSocket;

pub async fn listener() -> std::io::Result<()> {
    //open the port
    let socket = UdpSocket::bind("0.0.0.0:53").await?;
    println!("Listening on 0.0.0.0:53...");
    let mut buf = [0; 1024];

    loop {
        let (length, src) = socket.recv_from(&mut buf).await?;
        println!("Received {} bytes from {:?}", length, src);

        //Parse the packet as DNS
        if let Ok(packet) = Message::from_vec(&buf[..length]) {
            let header = packet.header();
            let quest = packet.queries();

            println!("DNS ID: {}", header.id());
            println!("Query: {:?}", quest);

            for q in quest {
                let domain_name = q.name();
                let record_type = q.query_type();
                println!("Domain: {}, Record Type: {:?}", domain_name, record_type);

                let all_addresses: Vec<IpAddr> =
                    if blocklookup::check_dn_block_list(domain_name.clone()) {
                        vec![IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))]
                    } else if record_type == RecordType::A {
                        // Query Operatin on record_type A
                        println!("Query Operatin on record_type A");
                        match resolver::resolve_domain(&domain_name.to_string()).await {
                            Ok(ips) => ips,
                            Err(e) => {
                                eprintln!("Failed to resolve the domain: {}", e);
                                vec![]
                            }
                        }
                    } else {
                        vec![]
                    };

                let payload =
                    parsing_dns_packet(header.id(), &domain_name, record_type, all_addresses);
                socket.send_to(&payload, src).await?;
            }
        } else {
            println!("this is an escape, there was a ERROR with the recived packet.");
        }
    }
}

pub fn parsing_dns_packet(
    id: u16,
    query_name: &Name,
    query_type: RecordType,
    addrs: Vec<IpAddr>,
) -> Vec<u8> {
    let ttl: u32 = 300;

    let mut query = Query::new();
    query
        .set_name(query_name.clone())
        .set_query_type(query_type);

    let mut response = Message::new();
    response
        .set_id(id)
        .set_message_type(MessageType::Response)
        .set_op_code(OpCode::Query)
        .set_recursion_available(true)
        .set_response_code(ResponseCode::NoError)
        .add_query(query);

    for ip in addrs {
        match ip {
            IpAddr::V4(v4) => {
                let answer = Record::from_rdata(query_name.clone(), ttl, RData::A(rdata::A(v4)));
                response.add_answer(answer);
            }
            IpAddr::V6(v6) => {
                let answer =
                    Record::from_rdata(query_name.clone(), ttl, RData::AAAA(rdata::AAAA(v6)));
                response.add_answer(answer);
            }
        }
    }

    // ----- reformating to bytes ------
    let mut buf: Vec<u8> = Vec::new();
    response
        .emit(&mut BinEncoder::new(&mut buf))
        .expect("failed at serialization");
    buf
}
