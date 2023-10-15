use serde::Serialize;
use std::error::Error;

use super::{
    active_connection::ActiveConnection,
    device::{self, Device},
};
use zbus;
use zbus::dbus_proxy;
use zbus::Connection;
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

#[derive(Debug, OwnedValue)]
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

pub struct NetworkManager<'a> {
    proxy: NetworkManagerProxy<'a>,
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
}
