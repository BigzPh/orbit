# Orbit 🪐
> **The lightweight Antigravity Mini Sidebar for macOS & Desktop**

Orbit is a native, ultra-lightweight ($<40\text{ MB}$ RAM) desktop companion app for **Google Antigravity**. It anchors directly to the side of your display so you can watch your agents think, review diffs, approve tool executions, and send prompts—all while keeping your primary IDE or browser in full view.

---

## ✨ Features

- **🎯 Auto Docking & Smooth Animation:** Slides smoothly onto the screen with native macOS CoreAnimation. Configurable to dock on the right or left edge.
- **⚡️ Global Hotkey Summon:** Press <kbd>⌥ Option</kbd> + <kbd>Space</kbd> (or <kbd>⌘</kbd> + <kbd>⇧ Shift</kbd> + <kbd>A</kbd>) anywhere in macOS to summon or dismiss Orbit instantly.
- **🌌 Multi-Space & Full-Screen Auxiliary:** Floats seamlessly across all virtual desktops and over native full-screen IDEs/browsers (`CanJoinAllSpaces | FullScreenAuxiliary`).
- **✍️ Immediate Typing Focus:** Type prompts immediately without focus fighting or window switching.
- **🪐 macOS Menu Bar Tray:**
  - One-click toggle from the top menu bar.
  - Dock to Right / Dock to Left edge toggles.
  - "Always on Top" toggle.
  - "Restart to Antigravity Home" (resets back if you ever get lost).
  - Quick reload (<kbd>⌘ R</kbd>), app relaunch, and browser fallback.
- **🔒 Native WebKit Performance:** Uses macOS WKWebView directly via Tauri v2, avoiding Electron's heavy memory overhead (<40MB RAM).
- **🔑 Full Google Auth & WebSockets:** Supports Google account sign-in, persistent session cookies, and live streaming agent updates.

---

## 📦 Installation

Download the latest release for Apple Silicon macOS:
- **[Download Orbit macOS (.dmg)](https://github.com/BigzPh/orbit/releases/latest)**

Mount the `.dmg` and drag `Orbit.app` to your `/Applications` folder.

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
npm run tauri build
```
The compiled macOS `.dmg` and `.app` bundles will be created under `src-tauri/target/release/bundle/`.

---

## ⌨️ Shortcuts Reference

| Shortcut / Action | Description |
| :--- | :--- |
| <kbd>⌥ Option</kbd> + <kbd>Space</kbd> | Global shortcut to summon / dismiss Orbit |
| <kbd>⌘</kbd> + <kbd>⇧ Shift</kbd> + <kbd>A</kbd> | Alternative global summon hotkey |
| <kbd>⌘</kbd> + <kbd>R</kbd> | Refresh the Antigravity Remote webview stream |
| **Menu Bar Icon (Left Click)** | Toggle sidebar visibility |
| **Menu Bar Icon (Right Click)** | Access settings (Restart to Home, Dock Side, Always on Top, Quit) |

---

## 🛠 Tech Stack

- **Framework:** Tauri v2
- **Backend:** Rust (`tauri`, `tauri-plugin-global-shortcut`, `tauri::tray`)
- **Webview Engine:** macOS Native WKWebView
- **Remote Host:** `https://antigravity.google.com` (configurable)
