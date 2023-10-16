mod access_point;
mod active_connection;
mod device;
mod ip4_config;
mod networkmanager;
mod ping;

use serde_json::json;
use std::error::Error;
use zbus::Connection;

pub(crate) async fn info() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let nm = networkmanager::NetworkManager::new(&connection).await?;

    let mut prop_stream = nm.proxy.receive_state_changed().await;
    while let Some(prop_changed) = prop_stream.next().await {
        let state = prop_changed.get().await?;
        // Get currently used connection
        let pc = nm.get_primary_connection().await?;
        // retrive data
        print_state(pc, state).await?;
    }

    Ok(())
}

enum Conn<'a> {
    Asleep,
    Connected {
        // Current interface/Device
        interface: &'a str,
        // Connecting, Connected, Disconnected, Disconnecting, Connected_Global, Connected_Local,
        state: &'a str,
        // Wireless, Wired
        kind: &'a str,
        // Only Wireless
        ssid: &'a str,
        // Only Wireless
        signal_strength: &'a str,
        // Only Wireless
        frequency: &'a str,
        // Wireless & Wirer
        ipaddr: &'a str,
        // Wireless & Wired
        cird: &'a str,
        // Wireless & Wired
        gateway: &'a str,
        // At any Time
        icon: &'a str,
    },
}

// impl<'a> TryFrom for Conn<'a> {
//     type Error = Box<dyn Error>;

//     fn try_from(value: T) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

async fn print_state(
    pc: active_connection::ActiveConnection<'_>,
    state: NMState,
) -> Result<(), Box<dyn Error>> {
    match tokio::join!(
        is_connected_to_internet(),
        async {
            let pc_devices = pc.get_devices().await?;
            let pc_device = pc_devices.first().unwrap();

            futures_util::future::try_join(pc_device.get_device_data(), pc_device.get_interface())
                .await
        },
        async {
            let pc_config = pc.get_ip4_config().await?;
            futures_util::future::try_join(pc_config.get_gateway(), pc_config.get_addresses()).await
        },
    ) {
        (global_conn, Ok((d_data, interface)), Ok((gateway, pc_config_addresses))) => {
            let pc_config_address = pc_config_addresses.first().unwrap();
            let ipaddr = &pc_config_address.address;
            let cidr = &pc_config_address.mask_cird;
            let (kind, frequency, signal_strength, ssid) = d_data;

            let state = if NMState::ConnectedGlobal == state && !global_conn {
                NMState::ConnectedLocal
            } else {
                state
            };
            let icon_path = icon_path(state, global_conn, signal_strength);
            
            let info = json!({
                "state": state.to_string(), // Connecting, Disconnected, Disconnecting, Connected_Global, Connected_Local,
                "interface": interface, // Current interface/Device
                "kind": kind, // Wireless, Wired
                "ssid": ssid, // Only Wireless
                "signalStrength": signal_strength, // Only Wireless
                "frequency": frequency, // Only Wireless
                "ipaddr": ipaddr, // Wireless & Wirer
                "cidr": cidr, // Wireless & Wired
                "gateway": gateway, // Wireless & Wired
                "icon": icon_path, // At any Time
            });
            Ok(println!("{}", info))
        }
        (_, _, _) => {
            Ok(println!(
                "{}",
                json!({
                    "state": state.to_string(), // Connecting, Connected, Disconnected, Disconnecting, ConnectedGlobal, ConnectedLocal, Asleep
                })
            ))
        }
    }
}
use async_io::Timer;
use futures_util::{
    future::select,
    future::Either::{Left, Right},
    pin_mut, StreamExt, TryStreamExt,
};
use zbus::{fdo::NameOwnerChanged, AsyncDrop, MatchRule, MessageStream};

use self::{
    networkmanager::{icon_path, NMState},
    ping::is_connected_to_internet,
};

pub async fn test() -> Result<(), Box<dyn Error>> {
    let conn = Connection::session().await?;
    let rule = MatchRule::builder()
        .msg_type(zbus::MessageType::Signal)
        .sender("org.freedesktop.DBus")?
        .interface("org.freedesktop.DBus")?
        .member("NameOwnerChanged")?
        .add_arg("org.freedesktop.zbus.MatchRuleStreamTest42")?
        .build();
    let mut stream = MessageStream::for_match_rule(
        rule,
        &conn,
        // For such a specific match rule, we don't need a big queue.
        Some(1),
    )
    .await?;

    let rule_str = "type='signal',sender='org.freedesktop.DBus',\
                interface='org.freedesktop.DBus',member='NameOwnerChanged',\
                arg0='org.freedesktop.zbus.MatchRuleStreamTest42'";
    assert_eq!(
        stream.match_rule().map(|r| r.to_string()).as_deref(),
        Some(rule_str),
    );

    // We register 2 names, starting with the uninteresting one. If `stream` wasn't filtering
    // messages based on the match rule, we'd receive method return call for each of these 2
    // calls first.
    //
    // Note that the `NameOwnerChanged` signal will not be sent by the bus  for the first name
    // we register since we setup an arg filter.
    conn.request_name("org.freedesktop.zbus.MatchRuleStreamTest44")
        .await?;
    conn.request_name("org.freedesktop.zbus.MatchRuleStreamTest42")
        .await?;

    let msg = stream.try_next().await?.unwrap();
    let signal = NameOwnerChanged::from_message(msg).unwrap();
    assert_eq!(
        signal.args()?.name(),
        "org.freedesktop.zbus.MatchRuleStreamTest42"
    );
    stream.async_drop().await;

    // Ensure the match rule is deregistered and this connection doesn't receive
    // `NameOwnerChanged` signals.
    let stream = MessageStream::from(&conn)
        .try_filter_map(|msg| async move { Ok(NameOwnerChanged::from_message(msg)) });
    conn.release_name("org.freedesktop.zbus.MatchRuleStreamTest42")
        .await?;

    pin_mut!(stream);
    let next = stream.try_next();
    pin_mut!(next);
    let timeout = Timer::after(std::time::Duration::from_millis(50));
    pin_mut!(timeout);
    match select(next, timeout).await {
        Left((msg, _)) => unreachable!("unexpected message: {:?}", msg),
        Right((_, _)) => (),
    };
    Ok(())
}
