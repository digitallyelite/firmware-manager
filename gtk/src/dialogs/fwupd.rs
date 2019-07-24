use crate::{ActivateEvent, Entity, FirmwareEvent, FirmwareInfo, FwupdDevice, FwupdRelease};
use super::{DialogData, FirmwareUpdateDialog};
use std::{collections::BTreeSet, sync::Arc};
use gtk::{self, prelude::*};

pub(crate) struct FwupdDialogData {
    pub entity:   Entity,
    pub device:   Arc<FwupdDevice>,
    pub releases: BTreeSet<FwupdRelease>,
    pub shared:   DialogData,
}

pub(crate) fn fwupd_dialog(data: &FwupdDialogData, upgradeable: bool, upgrade_button: bool) {
    let &FwupdDialogData { entity, device, releases, shared } = &data;
    let &DialogData { sender, tx_progress, stack, progress, info } = &shared;

    let response = if !upgrade_button || device.needs_reboot() {
        let &FirmwareInfo { ref latest, .. } = &info;

        let log_entries = releases
            .iter()
            .rev()
            .map(|release| (release.version.as_ref(), release.description.as_ref()));

        let dialog =
            FirmwareUpdateDialog::new(latest, log_entries, upgradeable, device.needs_reboot());

        let response = dialog.run();
        dialog.destroy();
        response
    } else {
        gtk::ResponseType::Accept.into()
    };

    if gtk::ResponseType::Accept == response {
        // Exchange the button for a progress bar.
        if let (Some(stack), Some(progress)) = (stack.upgrade(), progress.upgrade()) {
            stack.set_visible_child(&progress);
            let _ = tx_progress.send(ActivateEvent::Activate(progress));
        }

        let _ = sender.send(FirmwareEvent::Fwupd(
            *entity,
            device.clone(),
            Arc::new(releases.iter().last().expect("no release found").clone()),
        ));
    }
}