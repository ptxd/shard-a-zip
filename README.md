# Shard-A-Zip

<div align="center">

**A cross-platform, memory-efficient ZIP file splitter**

[![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/YOUR_USERNAME/shard-a-zip/releases)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/YOUR_USERNAME/shard-a-zip/releases)
[![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/YOUR_USERNAME/shard-a-zip/releases)

[**Download Latest Release**](https://github.com/YOUR_USERNAME/shard-a-zip/releases/latest)

</div>

---

## ✨ Features

- **📦 Smart Splitting** - Automatically splits ZIP files into chunks ≤25MB
- **🔀 Large File Support** - Files >25MB are split into binary parts
- **💾 Memory Efficient** - Streams data without loading entire files into memory
- **🖥️ Native UI** - Uses your OS's native file picker dialog
- **🌐 Cross-Platform** - Works on Windows, Linux, and macOS

---

## 📥 Quick Start

### Download

Go to [**Releases**](https://github.com/YOUR_USERNAME/shard-a-zip/releases/latest) and download for your platform:

| Platform | File |
|----------|------|
| Windows | `shard-a-zip-windows-x64.exe` |
| Linux | `shard-a-zip-linux-x64` |
| macOS (Intel) | `shard-a-zip-macos-x64` |
| macOS (Apple Silicon) | `shard-a-zip-macos-arm64` |

### Run

1. **Windows**: Double-click `shard-a-zip.exe`
2. **Linux/macOS**: 
   ```bash
   chmod +x shard-a-zip
   ./shard-a-zip
   ```

3. Select your ZIP file using the file dialog
4. Output files appear in the same folder as your source ZIP

---

## 📁 Output

- Files are named: `originalname1.zip`, `originalname2.zip`, etc.
- Each output file is ≤25MB
- A `_SHARD_A_ZIP_MANIFEST.txt` is included if any files were split into parts

---

## 🔧 Rejoining Split Files

If individual files were larger than 25MB, they're split into parts (`.part001`, `.part002`, etc.).

<details>
<summary><strong>Windows (PowerShell)</strong></summary>

```powershell
Get-Content file.ext.part* -Encoding Byte -ReadCount 0 | Set-Content file.ext -Encoding Byte
```
</details>

<details>
<summary><strong>Windows (CMD)</strong></summary>

```cmd
copy /b file.ext.part001+file.ext.part002+file.ext.part003 file.ext
```
</details>

<details>
<summary><strong>Linux/macOS</strong></summary>

```bash
cat file.ext.part* > file.ext
```
</details>

---

## 🛠️ Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) 1.70+

### Build
```bash
git clone https://github.com/YOUR_USERNAME/shard-a-zip.git
cd shard-a-zip
cargo build --release
```

Executable will be at `target/release/shard-a-zip` (or `.exe` on Windows).

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
