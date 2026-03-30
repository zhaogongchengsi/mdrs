use gpui::SharedString;
use gpui_component::IconNamed;

#[derive(Clone, Copy)]
pub enum AppIcon {
    Back,
    Edit,
    File,
    Folder,
    Menu,
    Preview,
    Settings,
}

impl IconNamed for AppIcon {
    fn path(self) -> SharedString {
        match self {
            Self::Back => "icons/back.svg",
            Self::Edit => "icons/edit.svg",
            Self::File => "icons/file.svg",
            Self::Folder => "icons/folder.svg",
            Self::Menu => "icons/menu.svg",
            Self::Preview => "icons/preview.svg",
            Self::Settings => "icons/settings.svg",
        }
        .into()
    }
}
