use gpui::App;

gpui::actions!(
    mdrs,
    [
        OpenFile,
        OpenFolder,
        SaveFile,
        OpenSettings,
        ToggleSidebar,
        Quit,
        // Placeholder action for menu items that are handled entirely by the OS
        // via OsAction (Cut, Copy, Paste, Undo, Redo, Select All).
        NoOp
    ]
);

pub fn register(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| {
        cx.quit();
    });
    // NoOp intentionally has no handler — the OsAction selector handles
    // the actual behaviour through the macOS responder chain.
}
