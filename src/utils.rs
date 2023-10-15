use std::{collections::HashMap, error::Error};
use zbus::{zvariant::Value, Connection};

pub(crate) async fn send_notification(connection: Connection) -> Result<(), Box<dyn Error>> {
    let m = connection
        .call_method(
            Some("org.freedesktop.Notifications"),
            "/org/freedesktop/Notifications",
            Some("org.freedesktop.Notifications"),
            "Notify",
            &(
                "my-app",
                0u32,
                "dialog-information",
                "A summary",
                "Some body",
                vec![""; 0],
                HashMap::<&str, &Value>::new(),
                5000,
            ),
        )
        .await?;
    let reply: u32 = m.body().unwrap();
    dbg!(reply);
    Ok(())
}

