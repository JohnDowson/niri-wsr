use std::{collections::BTreeMap, error::Error, sync::mpsc::channel, thread};

use niri_ipc::{socket::Socket, Action, Event, Request, WorkspaceReferenceArg};

fn main() -> Result<(), Box<dyn Error>> {
    let stream = {
        let sock = Socket::connect()?;
        let (resp, mut stream) = sock.send(Request::EventStream)?;
        resp?;

        let (tx, rx) = channel();

        ctrlc::set_handler({
            let tx = tx.clone();
            move || {
                _ = tx.send(None);
            }
        })?;

        thread::spawn(move || {
            while let Ok(ev) = stream() {
                _ = tx.send(Some(ev));
            }
        });
        rx
    };

    let mut state: BTreeMap<u64, BTreeMap<u64, String>> = BTreeMap::new();

    while let Ok(Some(ev)) = stream.recv() {
        match ev {
            Event::WorkspacesChanged { workspaces } => {
                let ids = workspaces.into_iter().map(|w| w.id).collect::<Vec<_>>();
                state.retain(|k, _| ids.contains(k));
                for w_id in ids {
                    state.entry(w_id).or_default();
                }
            }
            Event::WindowsChanged { windows } => {
                let ids = windows.iter().map(|w| w.id).collect::<Vec<_>>();
                for ws in state.values_mut() {
                    ws.retain(|k, _| ids.contains(k));
                }
                for window in windows {
                    let (Some(workspace_id), Some(mut title)) =
                        (window.workspace_id, window.app_id)
                    else {
                        continue;
                    };

                    truncate_window_title(&mut title);

                    state
                        .entry(workspace_id)
                        .or_default()
                        .insert(window.id, title);
                }
            }
            Event::WindowOpenedOrChanged { window } => {
                let (Some(workspace_id), Some(mut title)) =
                    (window.workspace_id, window.app_id.or(window.title))
                else {
                    continue;
                };

                truncate_window_title(&mut title);

                state
                    .entry(workspace_id)
                    .or_default()
                    .insert(window.id, title);
            }
            Event::WindowClosed { id } => {
                for ws in state.values_mut() {
                    ws.remove(&id);
                }
            }
            _ => (),
        }

        set_workspace_names(&state)?;
    }

    for workspace in state.keys() {
        let sender = Socket::connect()?;
        let (resp, _) = sender.send(Request::Action(Action::UnsetWorkspaceName {
            reference: Some(WorkspaceReferenceArg::Id(*workspace)),
        }))?;
        resp?;
    }

    Ok(())
}

fn set_workspace_names(state: &BTreeMap<u64, BTreeMap<u64, String>>) -> Result<(), Box<dyn Error>> {
    for (workspace, windows) in state.iter() {
        let name = windows
            .values()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("|");
        let sender = Socket::connect()?;

        let (resp, _) = if name.is_empty() {
            sender.send(Request::Action(Action::UnsetWorkspaceName {
                reference: Some(WorkspaceReferenceArg::Id(*workspace)),
            }))?
        } else {
            sender.send(Request::Action(Action::SetWorkspaceName {
                name,
                workspace: Some(WorkspaceReferenceArg::Id(*workspace)),
            }))?
        };
        resp?;
    }
    Ok(())
}

fn truncate_window_title(title: &mut String) {
    if title.len() > 12 {
        let (idx, _) = title
            .char_indices()
            .take_while(|(_, c)| !c.is_whitespace())
            .last()
            .unwrap();
        title.truncate(idx + 1);
        if idx > 12 {
            let (idx, _) = title.char_indices().take(8).last().unwrap();
            title.truncate(idx + 1);
            title.push('~')
        }
    }
}
