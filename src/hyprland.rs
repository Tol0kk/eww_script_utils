use hyprland::{
    data::{Devices, Workspace, Workspaces},
    event_listener::EventListener,
    shared::{HyprData, HyprDataActive},
};
use serde_json::{json, Value};
use std::error::Error;

/// Output a json whenever a change ocur
///
/// ``` json
/// {
///  "active_workspace":2,
///  "workspaces":[1,2],
/// }
///
/// ```
pub(crate) fn workspaces_listener() -> Result<(), Box<dyn Error>> {
    let mut event_listener = EventListener::new();

    println!("{}", serialize_workspaces()?);

    event_listener.add_workspace_added_handler(|_| match serialize_workspaces() {
        Err(_) => {}
        Ok(out) => println!("{}", out),
    });

    event_listener.add_workspace_change_handler(|_| match serialize_workspaces() {
        Err(_) => {}
        Ok(out) => println!("{}", out),
    });

    event_listener.add_workspace_destroy_handler(|_| match serialize_workspaces() {
        Err(_) => {}
        Ok(out) => println!("{}", out),
    });

    let _ = event_listener.start_listener();
    Ok(())
}

fn get_workspaces_id() -> Result<Vec<Value>, Box<dyn Error>> {
    let workspaces = Workspaces::get()?;
    let workspaces_id = workspaces
        .map(|workspace| {
            json!({
                "id": workspace.id,
                "windows": workspace.windows,
            })
        })
        .collect::<Vec<Value>>();
    Ok(workspaces_id)
}

fn get_workspaces_active_id() -> Result<i32, Box<dyn Error>> {
    let workspace = Workspace::get_active()?;
    Ok(workspace.id)
}

fn serialize_workspaces() -> Result<String, Box<dyn Error>> {
    let workspaces = get_workspaces_id()?;
    let active_workspace = get_workspaces_active_id()?;

    Ok(json!({
        "workspaces": workspaces,
        "active_workspace": active_workspace,
    })
    .to_string())
}

pub(crate) fn active_window_listener() -> Result<(), Box<dyn Error>> {
    let mut event_listener = EventListener::new();

    event_listener.add_active_window_change_handler(|a| {
        if let Some(active_window) = a {
            println!("{}", active_window.window_title)
        }
    });

    let _ = event_listener.start_listener();
    Ok(())
}

pub(crate) fn keyboard_language_listener() -> Result<(), Box<dyn Error>> {
    let mut event_listener = EventListener::new();
    let keybord_devices = Devices::get()?.keyboards;

    let kb = keybord_devices.iter().filter(|kb| kb.name.contains("(kb)")).min_by(|kb1,kb2| {
        kb1.name.cmp(&kb2.name)
    }).unwrap();
    println!("{}", kb.active_keymap);

    event_listener.add_keyboard_layout_change_handler(|layout| {
        println!("{}", layout.keyboard_name.split_once(',').unwrap().1);
    });

    let _ = event_listener.start_listener();
    Ok(())
}
