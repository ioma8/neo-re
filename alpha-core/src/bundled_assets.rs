use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundledCatalog {
    pub applets: Vec<BundledApplet>,
    pub os_images: Vec<BundledOsImage>,
}

impl BundledCatalog {
    pub fn dev_defaults() -> Self {
        let mut applets = stock_applets();
        applets.push(BundledApplet {
            id: "alpha-usb".to_owned(),
            applet_id: Some(0xa130),
            name: "Alpha USB".to_owned(),
            version: None,
            size: Some(ALPHA_USB_APPLET.len() as u64),
            kind: BundledAppletKind::AlphaUsb,
            source: BundledSource::Embedded {
                name: "alpha-usb-native.os3kapp",
                bytes: ALPHA_USB_APPLET,
            },
        });
        applets.sort_by(|left, right| left.name.cmp(&right.name));

        Self {
            applets,
            os_images: vec![BundledOsImage {
                id: "neo-os".to_owned(),
                name: "AlphaSmart NEO OS".to_owned(),
                kind: BundledOsImageKind::System,
                source: BundledSource::Embedded {
                    name: "os3kneorom.os3kos",
                    bytes: NEO_OS_IMAGE,
                },
            }],
        }
    }

    pub fn os_image_by_kind(&self, kind: BundledOsImageKind) -> Option<&BundledOsImage> {
        self.os_images.iter().find(|image| image.kind == kind)
    }

    pub fn original_stock_restore_applets(&self) -> Vec<&BundledApplet> {
        const RESTORE_ORDER: &[u16] = &[
            0xa000, 0xaf00, 0xaf75, 0xaf02, 0xaf73, 0xaf03, 0xa004, 0xa007, 0xa006, 0xa001,
            0xa002, 0xa027, 0xa005,
        ];

        RESTORE_ORDER
            .iter()
            .filter_map(|id| {
                self.applets
                    .iter()
                    .find(|applet| applet.kind == BundledAppletKind::Stock && applet.applet_id == Some(*id))
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundledApplet {
    pub id: String,
    pub applet_id: Option<u16>,
    pub name: String,
    pub version: Option<String>,
    pub size: Option<u64>,
    pub kind: BundledAppletKind,
    pub source: BundledSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundledAppletKind {
    Stock,
    AlphaUsb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundledOsImage {
    pub id: String,
    pub name: String,
    pub kind: BundledOsImageKind,
    pub source: BundledSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundledOsImageKind {
    Firmware,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundledSource {
    DevPath(PathBuf),
    Embedded {
        name: &'static str,
        bytes: &'static [u8],
    },
}

impl BundledSource {
    pub fn is_resolvable_without_picker(&self) -> bool {
        match self {
            Self::DevPath(_) | Self::Embedded { .. } => true,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::DevPath(path) => Some(path),
            Self::Embedded { .. } => None,
        }
    }
}

const STOCK_APPLETS: &[(&str, u16, &[u8])] = &[
    (
        "0000.os3kapp",
        0x0000,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/0000.os3kapp"
        ),
    ),
    (
        "A000.os3kapp",
        0xa000,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A000.os3kapp"
        ),
    ),
    (
        "A001.os3kapp",
        0xa001,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A001.os3kapp"
        ),
    ),
    (
        "A002.os3kapp",
        0xa002,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A002.os3kapp"
        ),
    ),
    (
        "A004.os3kapp",
        0xa004,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A004.os3kapp"
        ),
    ),
    (
        "A005.os3kapp",
        0xa005,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A005.os3kapp"
        ),
    ),
    (
        "A006.os3kapp",
        0xa006,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A006.os3kapp"
        ),
    ),
    (
        "A007.os3kapp",
        0xa007,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A007.os3kapp"
        ),
    ),
    (
        "A027.os3kapp",
        0xa027,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/A027.os3kapp"
        ),
    ),
    (
        "AF00.os3kapp",
        0xaf00,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/AF00.os3kapp"
        ),
    ),
    (
        "AF02.os3kapp",
        0xaf02,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/AF02.os3kapp"
        ),
    ),
    (
        "AF03.os3kapp",
        0xaf03,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/AF03.os3kapp"
        ),
    ),
    (
        "AF73.os3kapp",
        0xaf73,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/AF73.os3kapp"
        ),
    ),
    (
        "AF75.os3kapp",
        0xaf75,
        include_bytes!(
            "../../exports/smartapplet-backups/20260425-forth-clean-reflash/AF75.os3kapp"
        ),
    ),
];

const ALPHA_USB_APPLET: &[u8] = include_bytes!("../../exports/applets/alpha-usb-native.os3kapp");
const NEO_OS_IMAGE: &[u8] = include_bytes!("../../analysis/cab/os3kneorom.os3kos");

fn stock_applets() -> Vec<BundledApplet> {
    STOCK_APPLETS
        .iter()
        .map(|(name, applet_id, bytes)| BundledApplet {
            id: format!("{applet_id:04x}"),
            applet_id: Some(*applet_id),
            name: name.trim_end_matches(".os3kapp").to_owned(),
            version: None,
            size: Some(bytes.len() as u64),
            kind: BundledAppletKind::Stock,
            source: BundledSource::Embedded { name, bytes },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_contains_alpha_usb() {
        let catalog = BundledCatalog::dev_defaults();
        assert!(
            catalog
                .applets
                .iter()
                .any(|item| item.kind == BundledAppletKind::AlphaUsb)
        );
    }

    #[test]
    fn stock_workflows_do_not_require_user_paths() {
        let catalog = BundledCatalog::dev_defaults();
        assert!(
            catalog
                .applets
                .iter()
                .all(|item| item.source.is_resolvable_without_picker())
        );
        assert!(
            catalog
                .os_images
                .iter()
                .all(|item| item.source.is_resolvable_without_picker())
        );
    }

    #[test]
    fn bundled_catalog_has_unique_known_applet_ids() {
        let catalog = BundledCatalog::dev_defaults();
        let ids = catalog
            .applets
            .iter()
            .filter_map(|item| item.applet_id)
            .collect::<Vec<_>>();
        let unique = ids.iter().copied().collect::<BTreeSet<_>>();

        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn catalog_exposes_system_image_but_no_firmware_image() {
        let catalog = BundledCatalog::dev_defaults();

        assert!(
            catalog
                .os_image_by_kind(BundledOsImageKind::System)
                .is_some()
        );
        assert!(
            catalog
                .os_image_by_kind(BundledOsImageKind::Firmware)
                .is_none()
        );
    }

    #[test]
    fn original_stock_restore_order_excludes_alpha_usb_and_system() {
        let catalog = BundledCatalog::dev_defaults();
        let ordered = catalog.original_stock_restore_applets();
        let ids = ordered
            .iter()
            .filter_map(|item| item.applet_id)
            .collect::<Vec<_>>();

        assert!(!ids.contains(&0xa130), "Alpha USB must not be restored here");
        assert!(!ids.contains(&0x0000), "System applet must not be restored here");
        assert_eq!(
            ids,
            vec![
                0xa000, 0xaf00, 0xaf75, 0xaf02, 0xaf73, 0xaf03, 0xa004, 0xa007, 0xa006,
                0xa001, 0xa002, 0xa027, 0xa005,
            ]
        );
    }

    #[test]
    fn original_stock_restore_order_is_resolvable_without_picker() {
        let catalog = BundledCatalog::dev_defaults();

        assert!(
            catalog
                .original_stock_restore_applets()
                .iter()
                .all(|item| item.kind == BundledAppletKind::Stock
                    && item.source.is_resolvable_without_picker())
        );
    }
}
