use super::{
    active_connection::ActiveConnection,
    device::{self, Device},
};
use std::{error::Error, fmt::Display};
use zbus::dbus_proxy;
use zbus::Connection;
use zbus::{self, PropertyStream};
use zvariant::{OwnedObjectPath, OwnedValue};

#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NetworkManager {
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
    fn enable(&self, state: bool) -> zbus::Result<()>;
    #[dbus_proxy(property)]
    fn set_wireless_enabled(&self, state: bool) -> zbus::Result<()>;
    #[dbus_proxy(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn wireless_hardware_enabled(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn state(&self) -> zbus::Result<NMState>;
    #[dbus_proxy(property)]
    fn primary_connection_type(&self) -> zbus::Result<String>;
    #[dbus_proxy(property)]
    fn primary_connection(&self) -> zbus::Result<OwnedObjectPath>;
    #[dbus_proxy(property)]
    fn active_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[derive(Debug, OwnedValue, Clone, Copy, PartialEq, Eq)]
pub enum NMState {
    Unknow = 0,
    Asleep = 10,
    Disconnected = 20,
    Disconnecting = 30,
    Connecting = 40,
    ConnectedLocal = 50,
    ConnectedSite = 60,
    ConnectedGlobal = 70,
}

impl Display for NMState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct NetworkManager<'a> {
    pub(crate) proxy: NetworkManagerProxy<'a>,
    connection: &'a Connection,
}

impl NetworkManager<'_> {
    pub async fn new(connection: &Connection) -> Result<NetworkManager<'_>, Box<dyn Error>> {
        Ok(NetworkManager {
            proxy: NetworkManagerProxy::new(&connection).await?,
            connection: &connection,
        })
    }
    pub async fn get_devices(&self) -> Result<Vec<device::Device>, Box<dyn Error>> {
        let devices_path = self.proxy.get_devices().await?;
        let mut devices = vec![];
        for device_path in devices_path {
            devices.push(Device::new(device_path, &self.connection).await?)
        }
        Ok(devices)
    }
    pub async fn set_enable(&self, state: bool) -> Result<(), Box<dyn Error>> {
        self.proxy.enable(state).await?;
        Ok(())
    }
    pub async fn set_wireless_enabled(&self, state: bool) -> Result<(), Box<dyn Error>> {
        self.proxy.set_wireless_enabled(state).await?;
        Ok(())
    }
    pub async fn is_wireless_enable(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.proxy.wireless_enabled().await?)
    }
    pub async fn is_wireless_hardware_enable(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.proxy.wireless_hardware_enabled().await?)
    }
    pub async fn get_state(&self) -> Result<NMState, Box<dyn Error>> {
        Ok(self.proxy.state().await?)
    }
    pub async fn get_primary_connection_type(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.proxy.primary_connection_type().await?)
    }
    pub async fn get_primary_connection(&self) -> Result<ActiveConnection, Box<dyn Error>> {
        let connection_path = self.proxy.primary_connection().await?;
        Ok(ActiveConnection::new(connection_path, &self.connection).await?)
    }
    pub async fn get_active_connections(&self) -> Result<Vec<ActiveConnection>, Box<dyn Error>> {
        let connections_path = self
            .proxy
            .get_property::<Vec<OwnedObjectPath>>("Devices")
            .await?;
        let mut connections = vec![];
        for connection_path in connections_path {
            connections.push(ActiveConnection::new(connection_path, &self.connection).await?)
        }
        Ok(connections)
    }
    // pub async fn get_icon_path(&self, signal_strength: u8) -> Result<String, Box<dyn Error>> {
    //     let icon = NetworkState::new(
    //         self.get_state().await?,
    //         signal_strength,
    //         is_connected_to_internet().await,
    //     );
    //     Ok(icon_path() )

    // }
    pub async fn receive_property_changed(
        &self,
    ) -> Result<PropertyStream<NMState>, Box<dyn Error>> {
        // let receive_state_changed = self.proxy.receive_state_changed();
        // let mut b = receive_state_changed.boxed();
        // let c = b.next().await.ok_or("no prp")?;
        // let d = c.get().await?;
        todo!()
    }
}
pub fn icon_path(state: NMState, global_conn: bool, signal_strength: u8) -> String {
    NetworkState::new(state, signal_strength, global_conn).path()
}

pub enum NetworkState {
    Alseep,
    Disconnected,
    /// GIF
    Connecting,
    Disconnecting,
    Connected(u8),
    ConnectedGlobal(u8),
}
impl NetworkState {
    pub fn new(state: NMState, signal_strength: u8, is_connected: bool) -> NetworkState {
        match state {
            NMState::Unknow | NMState::Asleep => NetworkState::Alseep,
            NMState::Connecting => NetworkState::Connecting,
            NMState::Disconnecting => NetworkState::Disconnecting,
            NMState::ConnectedGlobal | NMState::ConnectedSite | NMState::ConnectedLocal => {
                match is_connected {
                    true => NetworkState::ConnectedGlobal(signal_strength),
                    false => NetworkState::Connected(signal_strength),
                }
            }
            NMState::Disconnected => NetworkState::Disconnected,
        }
    }
    pub fn path(&self) -> String {
        match self {
            NetworkState::Alseep => "/image/Asleep.svg",
            NetworkState::Disconnected => "/image/Disconnected.svg",
            NetworkState::Connecting => "/image/Connecting.gif",
            NetworkState::Disconnecting => "/image/Disconnecting.svg",
            NetworkState::Connected(x) => match x {
                x if *x < 25 => "/image/Connected-1.svg",
                x if 25 <= *x && *x < 50 => "/image/Connected-2.svg",
                x if 50 <= *x && *x < 75 => "/image/Connected-3.svg",
                x if 75 <= *x && *x <= 100 => "/image/Connected-4.svg",
                _ => panic!("Can't have a signal strengh over 100"),
            },
            NetworkState::ConnectedGlobal(x) => match x {
                x if *x < 25 => "/image/ConnectedGlobal-1.svg",
                x if 25 <= *x && *x < 50 => "/image/ConnectedGlobal-2.svg",
                x if 50 <= *x && *x < 75 => "/image/ConnectedGlobal-3.svg",
                x if 75 <= *x && *x <= 100 => "/image/ConnectedGlobal-4.svg",
                _ => panic!("Can't have a signal strengh over 100"),
            },
        }
        .to_string()
    }
}
