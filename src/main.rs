mod dbus;
use dbus::device::MouseDevice;
use zbus::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let devices = MouseDevice::load(&conn).await?;

    if devices.is_empty() {
        println!("No devices found. Is ratbagd running?");
        return Ok(());
    }

    for dev in &devices {
        println!("Device: {}", dev.name);
        println!("  DPI: {}", dev.dpi);
        println!("  Valid DPIs: {:?}", dev.valid_dpis);
        for btn in &dev.buttons {
            println!("  Button {}: {}", btn.index, btn.action.label());
        }
    }

    Ok(())
}