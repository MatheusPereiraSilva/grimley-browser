# 🧠 Grimley Browser

> Building a browser from scratch to understand the web at its core.

An experimental browser built with **Rust**, focused on deeply understanding how browsers work under the hood — from rendering and navigation to future features like privacy control and native ad blocking.

---

## 🚀 Purpose

Grimley Browser is not just another browser.

This project aims to:

* Understand how browsers actually work internally
* Control navigation programmatically
* Build a custom privacy system
* Implement a native ad blocker (Grimley Shield 🛡️)
* Serve as a learning and experimentation platform

---

## 🧩 Current Features

* ✅ Tab system
* ✅ Page navigation
* ✅ Functional URL bar
* ✅ Navigation history
* ✅ Basic PDF detection
* ✅ IPC communication (Rust ↔ WebView)
* ✅ UI synchronization with internal state

---

## 🛠️ Tech Stack

* 🦀 Rust
* 🪟 WebView2 (via Wry)
* 🧱 Tao (window & event loop)
* 🌐 HTML + JavaScript (embedded UI)

---

## 📦 Project Structure

```
src/
 ├── app/        # Main orchestration
 ├── browser/    # Navigation & WebView logic
 ├── ui/         # Window creation
 ├── tabs.rs     # Tab system
 ├── history.rs  # Navigation history
 └── main.rs     # Entry point
```

---

## ▶️ Getting Started

### 1. Install Rust

https://www.rust-lang.org/tools/install

---

### 2. Clone the repository

```
git clone https://github.com/MatheusPereiraSilva/grimley-browser.git
cd grimley-browser
```

---

### 3. Run the project

```
cargo run
```

---

## ⚠️ Important Notes

* This project is still under active development
* Some websites may not behave perfectly (WebView limitations)
* UI and performance are still being optimized

---

## 🛣️ Roadmap

### 🔜 Short-term

* [ ] Improve toolbar performance (reduce unnecessary re-renders)
* [ ] Optimize navigation flow
* [ ] Better WebView lifecycle management

---

### 🛡️ Grimley Shield (Future)

* [ ] Built-in HTTP proxy
* [ ] Ad blocking (EasyList support)
* [ ] Tracker blocking
* [ ] Cookie control
* [ ] Anti-fingerprinting system

---

### 🧠 Advanced Features

* [ ] Extension system
* [ ] Plugin architecture
* [ ] Session management
* [ ] Custom DevTools

---

## 🤝 Contributing

This is an open-source project focused on learning and experimentation.

Feel free to:

* Open issues
* Suggest improvements
* Submit pull requests

---

## 📄 License

MIT

---

## 👨‍💻 Author

Matheus Pereira
