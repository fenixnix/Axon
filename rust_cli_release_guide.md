# Rust 项目 GitHub Actions 全自动化 Release 方案

> 基于原文档修正与补充，原文有 Action 名称错误、缺少缓存等问题。

---

## 问题清单

| 问题 | 原内容 | 修正 |
|------|--------|------|
| Action 名错误 | `dtolnay/rust-action` | → `dtolnay/rust-toolchain` |
| 缺少缓存 | 无 | → 加 `swatinem/rust-cache@v2` |
| 缺少 CI 测试 | 只有 release | → 先 test 再 build |
| 只有 x86_64 | 漏了 ARM64 | → 补充 aarch64 和 Apple Silicon |

---

## 方案一：推荐 - cargo-dist (自动处理多平台)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --all-features

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

  release:
    needs: ci
    permissions:
      contents: write
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install cargo-dist
        run: curl -LsSL https://raw.githubusercontent.com/axodotdev/cargo-dist/main/install.sh | sh

      - name: Run cargo-dist
        run: cargo dist build --all
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**功能**: 自动生成多平台二进制 + Homebrew/Scoop 支持 + 安装脚本。

---

## 方案二：手动配置 (完全控制)

```yaml
name: Release Build

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test --all-features

  build:
    needs: test
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            ext: tar.gz
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            ext: tar.gz
          - os: macos-latest
            target: x86_64-apple-darwin
            ext: tar.gz
          - os: macos-latest
            target: aarch64-apple-darwin
            ext: tar.gz
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            ext: zip

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: swatinem/rust-cache@v2

      - name: Build release
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package
        shell: bash
        run: |
          BIN_NAME="your-binary-name"
          EXT="${{ matrix.ext }}"
          cd target/${{ matrix.target }}/release

          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            powershell -Command "Compress-Archive -Path ${BIN_NAME}.exe -DestinationPath ../../../${BIN_NAME}-${GITHUB_REF#refs/tags/v}-${TARGET}.${EXT}"
          else
            tar czvf ../../../${BIN_NAME}-${GITHUB_REF#refs/tags/v}-${TARGET}.${EXT} ${BIN_NAME}
          fi
        env:
          TARGET: ${{ matrix.target }}
          GITHUB_REF: ${{ github.ref }}

      - uses: softprops/action-gh-release@v2
        with:
          files: |
            *.tar.gz
            *.zip
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 发布命令

```bash
# 1. 提交 Cargo.lock
git add Cargo.lock
git commit -m "chore: lock file"
git push

# 2. 打标签 (必须与 Cargo.toml version 一致)
git tag v0.1.0
git push origin v0.1.0

# 3. GitHub Actions 自动执行: test → build → release
# 4. 去 Releases 页面点击 Publish
```

---

## crates.io 自动发布 (可选)

```yaml
crates-io:
  needs: test
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: swatinem/rust-cache@v2
    - run: cargo publish
      env:
        CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```
