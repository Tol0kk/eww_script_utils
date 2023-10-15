use std::error::Error;

use zbus::{Connection, Proxy};
use zvariant::OwnedObjectPath;


#[derive(Debug)]
pub struct AccessPoints<'a> {
    path: String,
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl AccessPoints<'_> {
    pub async fn new<'a>(
        path: OwnedObjectPath,
        connection: &'a Connection,
    ) -> Result<AccessPoints<'a>, Box<dyn Error>> {
        let p = Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path.clone(),
            "org.freedesktop.NetworkManager.AccessPoint",
        )
        .await?;
        Ok(AccessPoints {
            path: path.to_string(),
            proxy: p,
            connection,
        })
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub async fn get_ssid(&self) -> Result<String, Box<dyn Error>> {
        let ssid_raw = self.proxy.get_property::<Vec<u8>>("Ssid").await?;
        Ok(std::str::from_utf8(&ssid_raw)?.to_string())
    }
    /// The radio channel frequency in use by the access point, in MHz.
    pub async fn get_frequency(&self) -> Result<u32, Box<dyn Error>> {
        Ok(self.proxy.get_property::<u32>("Frequency").await?)
    }
    /// The maximum bitrate this access point is capable of, in kilobits/second (Kb/s).
    pub async fn get_max_bitrate(&self) -> Result<u32, Box<dyn Error>> {
        Ok(self.proxy.get_property::<u32>("MaxBitrate").await?)
    }
    pub async fn get_strength(&self) -> Result<u8, Box<dyn Error>> {
        Ok(self.proxy.get_property::<u8>("Strength").await?)
    }

}
