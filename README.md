# 🚀 Justfiles

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" />
  <img src="https://img.shields.io/badge/Platform-Linux-lightgrey?style=for-the-badge&logo=linux" />
  <img src="https://img.shields.io/badge/UI-Ratatui-red?style=for-the-badge" />
  <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" />
</p>

**Justfiles** — это минималистичный, быстрый и эстетичный файловый менеджер для терминала. Создан для тех, кто устал от перегруженных интерфейсов и хочет полного контроля через клавиатуру. 

Бросайте вызов скучным проводникам — переходите на **Vim-style** управление и чистый Rust.

---

## 🔥 Основные фишки

* 🦀 **Blazing Fast**: Написан на Rust, работает мгновенно даже в тяжелых директориях.
* ⌨️ **Vim-inspired**: Полное управление через командный режим (`:`).
* 🛡️ **Smart Preview**: Умный предпросмотр текстовых файлов с защитой от бинарного "мусора".
* 🎨 **Modern Aesthetic**: Поддержка иконок (Nerd Fonts) и стильная цветовая палитра (Aqua/Lavender/Pink).
* 🚫 **Ghost-Free**: Уникальная система очистки строк предотвращает появление графических артефактов.

## 🛠 Технологии

- **Core:** [Rust](https://www.rust-lang.org/)
- **TUI Framework:** [Ratatui](https://ratatui.rs/)
- **Backend:** [Crossterm](https://github.com/crossterm-rs/crossterm)

## 🚀 Быстрый старт

clone repo:

```bash
git clone https://github.com/Minish777/JustFiles
cd justfiles
```
build:

```bash
cargo build --release
```

add to path and START!

```bash
cargo install --path .
justfiles
```
