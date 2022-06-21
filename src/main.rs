use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use clap::Parser;
use rand;
use trust_dns::op::{Message, MessageType, OpCode, Query};
use trust_dns::rr::domain::Name;
use trust_dns::rr::record_type::RecordType;
use trust_dns::serialize::binary::*;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 's', long, default_value = "1.1.1.1")]
    dns_server: String,
    #[clap(long)]
    domain_name: String,
}

fn main() {
    let cli = Cli::parse();
    let dns_server: SocketAddr = format!("{}:53", cli.dns_server)
        .parse()
        .expect("Unable to parse dns_server as SocketAddr");
    let mut request_as_byte: Vec<u8> = Vec::with_capacity(512); // length of 0 and capacity of 512
    let mut response_as_byte: Vec<u8> = vec![0; 512]; // length of 512 and capacity of 512
    let domain_name = Name::from_ascii(&cli.domain_name).expect("Unable to parse the domain name");

    let mut msg = Message::new();
    msg
        .set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .add_query(Query::query(domain_name, RecordType::A))
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    let mut encoder = BinEncoder::new(&mut request_as_byte);
    msg.emit(&mut encoder).expect("Unable to emit the message");

    let localhost = UdpSocket::bind("0.0.0.0:0") // Listen all addresses on a random port selected by the OS
        .expect("Cannot bind to local socket");
    let timeout = Duration::from_secs(3);
    localhost.set_read_timeout(Some(timeout)).expect("Unable to listen network");
    localhost.set_nonblocking(false).expect("Unable to set the udp socket nonblocking to false");

    let _amt = localhost
        .send_to(&request_as_byte, dns_server)
        .expect("socket misconfigured");

    let (_amt, _remote) = localhost
        .recv_from(&mut response_as_byte)
        .expect("timeout reached");

    let dns_message = Message::from_vec(&response_as_byte)
        .expect("Unable to parse response");

    for answer in dns_message.answers() {
        if answer.record_type() == RecordType::A {
            let resource = answer.rdata();
            let ip = resource
                .to_ip_addr()
                .expect("Invalid IP address received");
            println!("{}", ip.to_string());
        }
    }

}
