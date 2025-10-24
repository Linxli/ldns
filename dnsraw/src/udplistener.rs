use hickory_proto::{
    op::{Message, MessageType, OpCode, Query, ResponseCode},
    rr::{Name, RData, Record, RecordType, rdata},
    serialize::binary::{BinEncodable, BinEncoder},
};
use std::net::IpAddr;
use tokio::net::UdpSocket;

use crate::resolver;

pub async fn listener() -> std::io::Result<()> {
    {
        //open the port
        let socket = UdpSocket::bind("127.0.0.1:34254").await?;
        println!("Listening on 127.0.0.1:34254...");
        let mut buf = [0; 1024];
        loop {
            let (length, src) = socket.recv_from(&mut buf).await?;
            println!("Received {} bytes from {:?}", length, src);

            //Parse the packet as DNS
            if let Ok(packet) = Message::from_vec(&buf[..length]) {
                let header = packet.header();
                let quest = packet.queries();

                println!("DNS ID: {}", header.id());

                for q in quest {
                    let domain_name = q.name();

                    if domain_name.is_empty() || domain_name.len() < 2 {
                        break;
                    };

                    let record_type = q.query_type();
                    println!("Domain: {}, Record Type: {:?}", domain_name, record_type);

                    if record_type == RecordType::A {
                        // Query Operatin on record_type A
                        println!("Query Operatin on record_type A");
                        let all_addresses =
                            resolver::resolve_domain(&domain_name.to_string()).await?;
                        //let payload = resolver::get_ip(all_addresses);
                        let payload = parsing_dns_packet(
                            header.id(),
                            &domain_name,
                            record_type,
                            all_addresses,
                        );
                        socket.send_to(&payload, src).await?;
                    }
                }
            } else {
                println!("this is an escape, there was a ERROR with the recived packet.");
            }

            // Redeclare `buf` as slice of the received data and send reverse data back to origin.
            buf.reverse();
            //println!("{:?} {:?}", buf, &src);
            socket.send_to(&buf, &src).await?;
        }
    } // the socket is closed here

    //Ok(())
}

#[allow(dead_code)]
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
