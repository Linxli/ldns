use dns_parser::Packet;
use hickory_proto::{op::Message, rr::Record, rr::Name};
use std::net::IpAddr;
use tokio::net::UdpSocket;

use crate::resolver;

pub async fn listener() -> std::io::Result<()> {
    {
        ///open the port
        let socket = UdpSocket::bind("127.0.0.1:34254").await?;
        println!("Listening on 127.0.0.1:34254...");
        let mut buf = [0; 1024];
        loop {
            let (length, src) = socket.recv_from(&mut buf).await?;
            println!("Received {} bytes from {:?}", length, src);

            ///Parse the packet as DNS
            if let Ok(packet) = Packet::parse(&buf[..length]) {
                let header = packet.header;
                let quest = packet.questions;
                let awnser = packet.answers;

                println!("DNS ID: {}", packet.header.id);

                for q in packet.questions {
                    let domain_name = q.qname;
                    let record_type = q.qtype;
                    println!("Domain: {}, Record Type: {:?}", domain_name, record_type);

                    if record_type == dns_parser::QueryType::A {
                        /// Query Operatin on record_type A
                        let all_addresses = resolver::resolve_DomainNmae(&domain_name.to_string()).await?;
                        let payload = parsing_DNS_packet(header.id, domain_name, all_addresses);
                        socket.send_to(payload, src).await?;
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

pub fn parsing_DNS_packet(id: u16, query_name: Name<>, addrs: Vec<IpAddr>) -> xxx {
    let response = Message::new();
    response
        .set_id(id)
        .set_message_type(MessageType::Response)
        .set_op_code(OpCode::Querry)
        .set_recursion_available(true)
        .set_response_code(ResponseCode::NoError)
        .add_query(Record::from_rdata(query_name.clone(), RecordType::A, RData::NULL));

    let awnser = Record::from_rdata(name, ttl, rdata)
}
