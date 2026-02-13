<div align='center'>
<h1>Hutch-Kick</h1>

<img src='src-tauri/icons/icon.png' width=200>
<p>Hutch-Kick is a lightweight Windows tray app that starts or stops a background service to "kick" your Hutch self-care page, while automatically pausing when Wi-Fi or internet drops. 📶⚡</p>
</div>

<br>
<div align="center">

## Downloads

| Platform | Download Link |
|----------|--------------|
| Windows (Setup) | [Hutch-Kick-x64-setup.exe](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick-tray_x64-setup.exe) |
| Windows (MSI) | [Hutch-Kick-x64_en-US.msi](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick-tray_x64_en-US.msi) |
| Windows (Portable) | [Hutch-Kick.exe](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick.exe) |
| Linux (AppImage) | [Hutch-Kick-amd64.AppImage](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick-tray_amd64.AppImage) |
| Linux (DEB) | [Hutch-Kick-amd64.deb](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick-tray_amd64.deb) |
| Linux (RPM) | [Hutch-Kick-x86_64.rpm](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick-tray-x86_64.rpm) |
| macOS (Apple Silicon) | [Hutch-Kick-aarch64.dmg](https://github.com/pamod-madubashana/Hutch-Kick/releases/latest/download/Hutch-Kick-tray_aarch64.dmg) |

</div>

## Tech Stack

- Frontend: React + TypeScript + Vite
- UI Components: shadcn/ui + Tailwind CSS
- Backend: Tauri (Rust)
- State Management: React Query

## Why This Project Exists

- Problem: Internet speed intermittently drops despite stable signal strength, impacting day-to-day browsing and app usage.
- Goal: Improve connection performance by periodically triggering a lightweight request to the Hutch Selfcare page to simulate an active session and keep the network state responsive.
- Outcome: A small Windows tray utility that can start/stop the booster service and automatically pauses when Wi-Fi or internet connectivity is lost, keeping behavior predictable and low-noise.


## Project Structure

```
src/
├── components/
│   ├── ui/              # shadcn/ui components
│   ├── AdvancedSettings.tsx
│   ├── LogPanel.tsx
│   ├── NavLink.tsx
│   ├── PopoverFooter.tsx
│   ├── PopoverHeader.tsx
│   ├── PrimaryControls.tsx
│   └── StatusCard.tsx
├── hooks/
│   └── useServiceState.ts
├── pages/
│   ├── Index.tsx
│   └── NotFound.tsx
├── App.tsx
├── main.tsx
└── index.css
src-tauri/
├── src/
│   ├── lib.rs
│   └── main.rs
├── Cargo.toml
├── tauri.conf.json
└── icons/
```

## Screenshots

<div align='center'>
    <img src='screenshots\1.png'>
    <img src='screenshots\2.png'>
    <img src='screenshots\3.png'>

</div>
## Key Features

- Runs in the system tray with quick access menu
- Automatically stops when Wi-Fi or internet is lost
- Always-on-top popover window for quick access
- Configurable kick interval (default: 20 seconds)
- Manual kick trigger at any time
- Real-time Wi-Fi and internet status display
- Activity logging for tracking service events

## Getting Started

### Prerequisites

- Node.js 18+
- Rust (latest stable)
- Windows 10/11

### Installation

```bash
# Clone the repository
git clone https://github.com/pamod-madubashana/Hutch-Kick.git

# Navigate to project directory
cd Hutch-Kick

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build for production
npm run tauri build
```

## Usage

1. Launch Hutch-Kick - it will appear in your system tray
2. Click the tray icon to open the control panel
3. Click **Start** to begin the kicking service
4. The service will automatically pause when Wi-Fi or internet drops
5. Use **Kick Now** to manually trigger a kick at any time
6. Configure the kick interval in **Advanced Settings**

## Keyboard Shortcuts

- **Left-click tray icon**: Show/hide control panel
- **Right-click tray icon**: Open context menu (Show/Hide, Start/Stop, Quit)
