use super::proxies::*;
use zbus::zvariant::Value;
use zbus::Connection;

#[derive(Debug, Clone)]
pub enum ButtonAction {
    None,
    Button(u32),
    Special(u32),
    Key(u32),
    Unknown,
}

impl ButtonAction {
    pub fn from_mapping(action_type: u32, value: &Value<'_>) -> Self {
        let v = match value {
            Value::Value(inner) => inner.as_ref(),
            other => other,
        };
        match action_type {
            0 => ButtonAction::None,
            1 => {
                if let Value::U32(n) = v { ButtonAction::Button(*n) } else { ButtonAction::Unknown }
            }
            2 => {
                if let Value::U32(n) = v { ButtonAction::Special(*n) } else { ButtonAction::Unknown }
            }
            3 => {
                if let Value::U32(n) = v { ButtonAction::Key(*n) } else { ButtonAction::Unknown }
            }
            _ => ButtonAction::Unknown,
        }
    }

    pub fn label(&self) -> String {
    match self {
        ButtonAction::None => "None".into(),
        ButtonAction::Button(n) => match n {
            1 => "Left Click".into(),
            2 => "Right Click".into(),
            3 => "Middle Click".into(),
            4 => "Back".into(),
            5 => "Forward".into(),
            6 => "Side Left".into(),
            7 => "Side Right".into(),
            8 => "Side Middle".into(),
            _ => format!("Button {}", n),
        },
        ButtonAction::Special(n) => format!("Special {}", n),
        ButtonAction::Key(n) => format!("Key {}", n),
        ButtonAction::Unknown => "Unknown".into(),
    }
}
}

#[derive(Debug, Clone)]
pub struct MouseButton {
    pub index: u32,
    pub action: ButtonAction,
    pub path: zbus::zvariant::OwnedObjectPath,
}

#[derive(Debug, Clone)]
pub struct MouseDevice {
    pub name: String,
    pub dpi: u32,
    pub valid_dpis: Vec<u32>,
    pub buttons: Vec<MouseButton>,
    pub device_path: zbus::zvariant::OwnedObjectPath,
    pub resolution_path: zbus::zvariant::OwnedObjectPath,
}

impl MouseDevice {
    pub async fn load(conn: &Connection) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let manager = ManagerProxy::new(conn).await?;
        let device_paths = manager.devices().await?;
        let mut devices = Vec::new();

        for device_path in device_paths {
            let device = DeviceProxy::builder(conn)
                .path(device_path.clone())?
                .build()
                .await?;

            let name = device.name().await?;

            for profile_path in device.profiles().await? {
                let profile = ProfileProxy::builder(conn)
                    .path(profile_path.clone())?
                    .build()
                    .await?;

                if !profile.is_active().await? {
                    continue;
                }

                // Resolution
                let mut dpi = 0u32;
                let mut valid_dpis = Vec::new();
                let mut resolution_path = profile.resolutions().await?.into_iter().next().unwrap();

                for res_path in profile.resolutions().await? {
                    let res = ResolutionProxy::builder(conn)
                        .path(res_path.clone())?
                        .build()
                        .await?;

                    if res.is_active().await? {
                        let raw = res.resolution().await?;
                        dpi = match &*raw {
                            Value::Value(inner) => match &**inner {
                                Value::U32(v) => *v,
                                _ => 0,
                            },
                            Value::U32(v) => *v,
                            _ => 0,
                        };
                        valid_dpis = res.resolutions().await?;
                        resolution_path = res_path;
                    }
                }

                // Buttons
                let mut buttons = Vec::new();
                for button_path in profile.buttons().await? {
                    let btn = ButtonProxy::builder(conn)
                        .path(button_path.clone())?
                        .build()
                        .await?;

                    let index = btn.index().await?;
                    let (action_type, raw_value) = btn.mapping().await?;
                    let action = ButtonAction::from_mapping(action_type, &raw_value);

                    buttons.push(MouseButton {
                        index,
                        action,
                        path: button_path,
                    });
                }

                devices.push(MouseDevice {
                    name: name.clone(),
                    dpi,
                    valid_dpis,
                    buttons,
                    device_path: device_path.clone(),
                    resolution_path,
                });
            }
        }

        Ok(devices)
    }

pub async fn set_dpi(&mut self, conn: &Connection, dpi: u32) -> Result<(), Box<dyn std::error::Error>> {
    let res = ResolutionProxy::builder(conn)
        .path(self.resolution_path.clone())?
        .build()
        .await?;

    res.set_resolution(
        zbus::zvariant::Value::Value(Box::new(zbus::zvariant::Value::U32(dpi).try_into().unwrap()))
    ).await?;

    let device = DeviceProxy::builder(conn)
        .path(self.device_path.clone())?
        .build()
        .await?;

    device.commit().await?;
    self.dpi = dpi;

    Ok(())
}
pub async fn set_button(
    &mut self,
    conn: &Connection,
    button_index: usize,
    action: ButtonAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let btn_path = self.buttons[button_index].path.clone();

    let btn = ButtonProxy::builder(conn)
        .path(btn_path)?
        .build()
        .await?;

    let (action_type, value) = match &action {
        ButtonAction::None => (0u32, zbus::zvariant::Value::U32(0)),
        ButtonAction::Button(n) => (1u32, zbus::zvariant::Value::U32(*n)),
        ButtonAction::Special(n) => (2u32, zbus::zvariant::Value::U32(*n)),
        ButtonAction::Key(n) => (3u32, zbus::zvariant::Value::U32(*n)),
        ButtonAction::Unknown => return Ok(()),
    };

    btn.set_mapping((action_type, value)).await?;

    let device = DeviceProxy::builder(conn)
        .path(self.device_path.clone())?
        .build()
        .await?;

    device.commit().await?;
    self.buttons[button_index].action = action;

    Ok(())
}
}