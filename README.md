# Orbit 🪐
> **The lightweight Antigravity Mini Sidebar for macOS & Desktop**

Orbit is a native, ultra-lightweight ($<40\text{ MB}$ RAM) desktop companion app for **Google Antigravity**. It anchors directly to the side of your display so you can watch your agents think, review diffs, approve tool executions, and send prompts—all while keeping your primary IDE or browser in full view.

---

## ✨ Features

- **🎯 Right-Edge Auto Docking:** Automatically positions itself flush against the right edge of your screen with full vertical workspace height.
- **⚡️ Global Hotkey Toggle:** Press <kbd>⌥ Option</kbd> + <kbd>Space</kbd> anywhere in macOS to summon or dismiss Orbit instantly.
- **📌 Always-on-Top:** Floats above your full-screen code editors, terminal windows, and browsers without stealing focus.
- **🪐 macOS Menu Bar Tray:**
  - One-click toggle from the top menu bar.
  - Dock to Right / Dock to Left edge toggles.
  - "Always on Top" toggle.
  - Quick reload (<kbd>⌘ R</kbd>) and browser fallback.
- **🔒 Native WebKit Performance:** Uses macOS WKWebView directly via Tauri v2, avoiding Electron's heavy memory overhead.
- **🔑 Full Google Auth & WebSockets:** Supports Google account sign-in, persistent session cookies, and live streaming agent updates.

---

## 🚀 Quick Start

### 1. Enable Remote Control on Your Workstation

Before opening Orbit, ensure Antigravity has Remote Control enabled:

* **In the Antigravity Desktop App:**
  Open Settings (`⌘ ,`) $\rightarrow$ **App** $\rightarrow$ Toggle **Enable Remote Control** to **On**.
* **In the Antigravity CLI:**
  Run `/remote-control` inside an active session, or launch with:
  ```bash
  agy --remote-control
  ```

### 2. Run Orbit in Development

```bash
# Install frontend CLI dependencies (if not already installed)
npm install

# Run the app locally
npm run dev
```

### 3. Build Production macOS App (`.app` / `.dmg`)

```bash
npm run build
```
The compiled macOS `.app` bundle will be created under `src-tauri/target/release/bundle/macos/Orbit.app`.

---

## ⌨️ Shortcuts Reference

| Shortcut / Action | Description |
| :--- | :--- |
| <kbd>⌥ Option</kbd> + <kbd>Space</kbd> | Global shortcut to show/hide the Orbit sidebar |
| <kbd>⌘</kbd> + <kbd>R</kbd> | Refresh the Antigravity Remote webview stream |
| **Menu Bar Icon (Left Click)** | Toggle sidebar visibility |
| **Menu Bar Icon (Right Click)** | Open settings menu (dock side, pin, quit) |
| **Close Button (Red Dot)** | Minimizes to tray instead of quitting |

---

## 🛠 Tech Stack

- **Framework:** Tauri v2
- **Backend:** Rust (`tauri`, `tauri-plugin-global-shortcut`, `tauri::tray`)
- **Webview Engine:** macOS Native WKWebView
- **Remote Host:** `https://antigravity.google.com` (configurable)
