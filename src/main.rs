use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use zbus::{proxy, Connection};

#[proxy(
    interface = "org.freedesktop.ratbag1.Manager",
    default_service = "org.freedesktop.ratbag1",
    default_path = "/org/freedesktop/ratbag1"
)]
trait Manager {
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Device",
    default_service = "org.freedesktop.ratbag1"
)]
trait Device {
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn profiles(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Profile",
    default_service = "org.freedesktop.ratbag1"
)]
trait Profile {
    #[zbus(property)]
    fn resolutions(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn buttons(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn is_active(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "org.freedesktop.ratbag1.Resolution",
    default_service = "org.freedesktop.ratbag1"
)]
trait Resolution {
    #[zbus(property)]
    fn resolution(&self) -> zbus::Result<OwnedValue>;

    #[zbus(property)]
    fn resolutions(&self) -> zbus::Result<Vec<u32>>;

    #[zbus(property)]
    fn is_active(&self) -> zbus::Result<bool>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let manager = ManagerProxy::new(&conn).await?;
    let devices = manager.devices().await?;

    if devices.is_empty() {
        println!("No devices found. Is ratbagd running?");
        return Ok(());
    }

    for device_path in devices {
        let device = DeviceProxy::builder(&conn)
            .path(device_path)?
            .build()
            .await?;

        let name = device.name().await?;
        println!("Device: {}", name);

        for (pi, profile_path) in device.profiles().await?.iter().enumerate() {
            let profile = ProfileProxy::builder(&conn)
                .path(profile_path.clone())?
                .build()
                .await?;

            if !profile.is_active().await? {
                continue;
            }

            println!("  Profile {} (active):", pi);

            for res_path in profile.resolutions().await? {
                let res = ResolutionProxy::builder(&conn)
                    .path(res_path)?
                    .build()
                    .await?;

                let active = res.is_active().await?;
                let dpi_raw = res.resolution().await?;
                let dpi: u32 = match &*dpi_raw {
                    Value::Value(inner) => match &**inner {
                        Value::U32(v) => *v,
                        _ => 0,
                    },
                Value::U32(v) => *v,
                _ => 0,
                };
                let valid_dpis = res.resolutions().await?;

                println!(
                    "    Resolution: {}dpi {}",
                    dpi,
                    if active { "<-- active" } else { "" }
                );
                println!("    Valid DPI values: {:?}", valid_dpis);
            }

            println!("  Button count: {}", profile.buttons().await?.len());
        }

        println!();
    }

    Ok(())
}
