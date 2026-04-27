use crate::{bundled_assets::BundledApplet, protocol::SmartAppletRecord};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppletChecklist {
    pub rows: Vec<AppletChecklistRow>,
}

impl AppletChecklist {
    pub fn from_installed_and_bundled(
        installed: &[SmartAppletRecord],
        bundled: &[BundledApplet],
    ) -> Self {
        let mut rows = installed
            .iter()
            .map(|record| {
                let bundled_match = bundled
                    .iter()
                    .find(|applet| applet.applet_id == Some(record.applet_id));
                AppletChecklistRow {
                    key: bundled_match
                        .map(|applet| applet.id.clone())
                        .unwrap_or_else(|| format!("installed-{:04x}", record.applet_id)),
                    display_name: record.name.clone(),
                    version: Some(record.version.clone()),
                    size: Some(record.file_size as u64),
                    installed: true,
                    checked: true,
                    source: bundled_match
                        .map(|applet| AppletSourceKind::Bundled {
                            id: applet.id.clone(),
                        })
                        .unwrap_or(AppletSourceKind::InstalledOnly {
                            applet_id: record.applet_id,
                        }),
                }
            })
            .collect::<Vec<_>>();

        for applet in bundled {
            let already_installed = rows.iter().any(|row| row.key == applet.id);
            if already_installed {
                continue;
            }
            rows.push(AppletChecklistRow {
                key: applet.id.clone(),
                display_name: applet.name.clone(),
                version: applet.version.clone(),
                size: applet.size,
                installed: false,
                checked: false,
                source: AppletSourceKind::Bundled {
                    id: applet.id.clone(),
                },
            });
        }
        rows.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Self { rows }
    }

    pub fn with_checked(mut self, key: &str, checked: bool) -> Self {
        if let Some(row) = self.rows.iter_mut().find(|row| row.key == key) {
            row.checked = checked;
        }
        self
    }

    pub fn plan(&self) -> AppletFlashPlan {
        let unchecked_installed = self
            .rows
            .iter()
            .filter(|row| row.installed && !row.checked)
            .count();
        let new_checked = self
            .rows
            .iter()
            .filter(|row| !row.installed && row.checked)
            .count();
        if unchecked_installed == 0 && new_checked == 0 {
            AppletFlashPlan::NoChanges
        } else if unchecked_installed == 0 {
            AppletFlashPlan::InstallOnly { count: new_checked }
        } else {
            AppletFlashPlan::ClearAndReinstall {
                count: self.rows.iter().filter(|row| row.checked).count(),
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppletChecklistRow {
    pub key: String,
    pub display_name: String,
    pub version: Option<String>,
    pub size: Option<u64>,
    pub installed: bool,
    pub checked: bool,
    pub source: AppletSourceKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppletSourceKind {
    InstalledOnly { applet_id: u16 },
    Bundled { id: String },
    AddedFromFile { path: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppletFlashPlan {
    NoChanges,
    InstallOnly { count: usize },
    ClearAndReinstall { count: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundled_assets::{BundledAppletKind, BundledSource};
    use std::path::PathBuf;

    fn installed() -> Vec<SmartAppletRecord> {
        vec![SmartAppletRecord {
            applet_id: 0xa000,
            version: "1.0".to_owned(),
            name: "AlphaWord".to_owned(),
            file_size: 1024,
            applet_class: 1,
        }]
    }

    fn bundled() -> Vec<BundledApplet> {
        vec![BundledApplet {
            id: "alpha-usb".to_owned(),
            applet_id: Some(0xa130),
            name: "Alpha USB".to_owned(),
            version: Some("0.1".to_owned()),
            size: Some(512),
            kind: BundledAppletKind::AlphaUsb,
            source: BundledSource::DevPath(PathBuf::from("alpha-usb.os3kapp")),
        }]
    }

    #[test]
    fn installed_applets_start_checked() {
        let state = AppletChecklist::from_installed_and_bundled(&installed(), &[]);
        assert!(state.rows.iter().all(|row| row.checked));
    }

    #[test]
    fn bundled_missing_applets_start_unchecked() {
        let state = AppletChecklist::from_installed_and_bundled(&installed(), &bundled());
        let row = state
            .rows
            .iter()
            .find(|row| row.key == "alpha-usb")
            .unwrap();
        assert!(!row.installed);
        assert!(!row.checked);
    }

    #[test]
    fn selecting_only_new_bundled_applet_is_install_only() {
        let state = AppletChecklist::from_installed_and_bundled(&installed(), &bundled());
        let changed = state.with_checked("alpha-usb", true);
        assert_eq!(changed.plan(), AppletFlashPlan::InstallOnly { count: 1 });
    }

    #[test]
    fn unchecking_installed_applet_requires_clear_reinstall() {
        let state = AppletChecklist::from_installed_and_bundled(&installed(), &bundled());
        let changed = state.with_checked("installed-a000", false);
        assert_eq!(
            changed.plan(),
            AppletFlashPlan::ClearAndReinstall { count: 0 }
        );
    }
}
