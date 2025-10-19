<div align="center">
  
<img src="src-tauri/icons/icon.png" alt="MapaNote Logo" width="120" height="120">

# MAPANOTE

**A local-first, privacy-focused desktop app for organizing geographic notes**

[![Download](https://img.shields.io/badge/Download-v0.1.0--alpha-blue?style=for-the-badge)](https://github.com/conpans/mapanote/releases/latest)
[![Website](https://img.shields.io/badge/Website-mapanote.com-green?style=for-the-badge)](https://mapanote.com)
[![License](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)](LICENSE)

[🌐 Website](https://mapanote.com) • [📥 Download](https://github.com/conpans/mapanote/releases/latest) • [🐛 Report Bug](https://github.com/conpans/mapanote/issues) • [💡 Request Feature](https://github.com/conpans/mapanote/issues)

---

### 🗺️ **Organize your geographic knowledge. Local-first. Privacy-focused. No accounts needed.**

</div>

---

## ✨ Features

- **📝 Rich Markdown Notes** - Full markdown support with images, code blocks, and formatting
- **🗺️ Country & Topic Organization** - Group notes by country and cross-cutting topics
- **💾 Local-First Storage** - All data stored on your machine in plain files
- **🔒 Privacy Focused** - No accounts, no cloud, no tracking, no telemetry
- **⚡ Fast & Lightweight** - Built with Rust and Tauri for minimal resource usage
- **🖼️ Image Support** - Embed images with drag-and-drop or paste
- **🏷️ Smart Tagging** - Tag and filter notes for easy discovery
- **📶 Works Offline** - No internet connection needed after download

## 🚀 Download

**Current Release:** v0.1.0-alpha (Windows only)

<a href="https://github.com/conpans/mapanote/releases/latest">
  <img src="https://img.shields.io/badge/Download_MapaNote-Windows-blue?style=for-the-badge&logo=windows" alt="Download for Windows">
</a>

> ⚠️ **Alpha Software** - Expect bugs and rough edges. Windows 10/11 only. macOS and Linux coming soon.

## 📸 Screenshots

_Coming soon_

## 🛠️ Tech Stack

- **Backend:** Rust + Tauri v2
- **Frontend:** Svelte 5 + SvelteKit + TypeScript
- **Styling:** TailwindCSS
- **Data Storage:** Local JSON files (SQLite for metadata)

## 🏗️ Development

### Prerequisites

- Rust (1.70+)
- Node.js (20 LTS)
- Visual Studio C++ Build Tools (Windows)

### Install

```bash
git clone https://github.com/conpans/mapanote.git
cd mapanote
npm install
```

### Run Development Server

```bash
npm run tauri dev
```

### Build for Production

```bash
npm run tauri build
```

Built files will be in `src-tauri/target/release/bundle/`

## 🐛 Known Issues

- **Map Coverage:** Some countries (France, North Macedonia, Somalia, and others) are not yet properly configured on the interactive SVG map. You can still create notes for these countries through the country list view.
- Windows-only in alpha - macOS and Linux builds coming soon

## 🤝 Contributing

Contributions welcome! This is an early alpha, so there's plenty to improve.

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Inspired by local-first knowledge management tools

---

<div align="center">

**[⬇️ Download MapaNote](https://github.com/conpans/mapanote/releases/latest)** | **[🌐 Visit Website](https://mapanote.com)**

Made with ❤️ for geography enthusiasts

</div>
