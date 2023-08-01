use futures::stream::StreamExt;
use futures::{executor::block_on, future::Either};
use libp2p::{
    core::multiaddr::Protocol,
    core::muxing::StreamMuxerBox,
    core::upgrade,
    core::{Multiaddr, Transport},
    identify, identity,
    identity::PeerId,
    noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp,
};
use libp2p_quic as quic;
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};

pub fn start_client(secret_key_seed: u8, port: u16, use_ipv6: bool) -> Result<(), Box<dyn Error>> {
    // Create a static known PeerId based on given secret
    let local_key: identity::Keypair = generate_ed25519(secret_key_seed);
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {local_peer_id:?}");

    let tcp_transport = tcp::async_io::Transport::default();

    let tcp_transport = tcp_transport
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(
            noise::Config::new(&local_key).expect("Signing libp2p-noise static DH keypair failed."),
        )
        .multiplex(libp2p::yamux::Config::default());

    let quic_transport = quic::async_std::Transport::new(quic::Config::new(&local_key));

    let transport = quic_transport
        .or_transport(tcp_transport)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        })
        .boxed();

    let behaviour = Behaviour {
        relay: relay::Behaviour::new(local_peer_id, Default::default()),
        ping: ping::Behaviour::new(ping::Config::new()),
        identify: identify::Behaviour::new(identify::Config::new(
            "/TODO/0.0.1".to_string(),
            local_key.public(),
        )),
    };

    let mut swarm = SwarmBuilder::without_executor(transport, behaviour, local_peer_id).build();

    // Listen on all interfaces
    let listen_addr_tcp = Multiaddr::empty()
        .with(match use_ipv6 {
            true => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(port));
    swarm.listen_on(listen_addr_tcp)?;

    let listen_addr_quic = Multiaddr::empty()
        .with(match use_ipv6 {
            true => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Udp(port))
        .with(Protocol::QuicV1);
    swarm.listen_on(listen_addr_quic)?;

    block_on(async {
        loop {
            match swarm.next().await.expect("Infinite Stream.") {
                SwarmEvent::Behaviour(event) => {
                    if let BehaviourEvent::Identify(identify::Event::Received {
                        info: identify::Info { observed_addr, .. },
                        ..
                    }) = &event
                    {
                        swarm.add_external_address(observed_addr.clone());
                    }

                    println!("{event:?}")
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {address:?}");
                }
                _ => {}
            }
        }
    })
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}
