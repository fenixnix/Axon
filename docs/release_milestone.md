# GitHub Actions CI/CD 发布流程 - 里程碑记录

> **日期**: 2026-03-18
> **项目**: Axon - Rust CLI Agent
> **成就**: 首次成功实现 Win + Ubuntu 双平台自动发布

---

## 最终成功的 Workflow 配置

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always
  BIN_NAME: axon

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - uses: dtolnay/rust-toolchain@stable
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
    needs: ci
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            ext: tar.gz
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            ext: zip

    steps:
      - uses: actions/checkout@v5

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: swatinem/rust-cache@v2

      - name: Build release
        run: cargo build --release --locked --target ${{ matrix.target }}

      - name: Verify binary exists
        shell: bash
        run: |
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            BIN_PATH="target/${{ matrix.target }}/release/${{ env.BIN_NAME }}.exe"
          else
            BIN_PATH="target/${{ matrix.target }}/release/${{ env.BIN_NAME }}"
          fi
          if [[ ! -f "$BIN_PATH" ]]; then
            echo "Binary not found at: $BIN_PATH"
            ls -la target/${{ matrix.target }}/release/
            exit 1
          fi
          echo "Binary found: $BIN_PATH"

      - name: Package
        shell: bash
        run: |
          cd target/${{ matrix.target }}/release
          EXT="${{ matrix.ext }}"

          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            powershell -Command "Compress-Archive -Path axon.exe -DestinationPath ../../../axon-${{ github.ref_name }}-${{ matrix.target }}.${EXT}"
          else
            tar czvf ../../../axon-${{ github.ref_name }}-${{ matrix.target }}.${EXT} ./axon
          fi

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ env.BIN_NAME }}-${{ matrix.target }}
          path: ${{ env.BIN_NAME }}-*${{ matrix.ext }}

  release:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          pattern: ${{ env.BIN_NAME }}-*

      - name: Create checksums
        run: |
          cd artifacts
          find . -type f -name "*.zip" -o -name "*.tar.gz" | xargs sha256sum > checksums.txt

      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          files: artifacts/**/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 血泪教训 - 踩坑记录

### 1. rust-version 与 edition 不匹配
| 问题 | 原因 | 解决 |
|------|------|------|
| `rust-version 1.75` 但 `edition = 2024` | edition 2024 需要 Rust 1.85+ | 修改 `Cargo.toml`: `rust-version = "1.85"` |

### 2. Cargo.lock 未提交
| 问题 | 原因 | 解决 |
|------|------|------|
| `--locked` 失败 | `.gitignore` 忽略了 `Cargo.lock` | 从 `.gitignore` 移除 `Cargo.lock` 并提交 |

### 3. clippy 警告 (跨平台差异)
| 问题 | 原因 | 解决 |
|------|------|------|
| Windows 端 `.args(&["/c", command])` | Linux clippy 更严格 | 去掉引用: `.args(["/c", command])` |

### 4. GitHub Actions Node.js 版本
| 问题 | 原因 | 解决 |
|------|------|------|
| Node.js 20 弃用警告 | actions/checkout@v4 | 升级到 `actions/checkout@v5` |

### 5. 打包脚本路径问题
| 问题 | 原因 | 解决 |
|------|------|------|
| `tar: Cowardly refusing to create an empty archive` | 二进制构建失败但继续执行 | 添加 `Verify binary exists` 步骤 |
| Windows 打包失败 | PowerShell 路径问题 | 使用绝对路径和 `github.ref_name` |

### 6. checksums 生成失败
| 问题 | 原因 | 解决 |
|------|------|------|
| `Is a directory` 错误 | download-artifact 默认下载目录结构 | 添加 `pattern` 过滤 + `find` 查找文件 |

---

## 使用方法

```bash
# 1. 确保 Cargo.lock 已提交
git add Cargo.lock
git commit -m "chore: lock file"
git push

# 2. 打标签（必须与 Cargo.toml version 一致）
git tag v0.1.0
git push origin v0.1.0

# 3. GitHub Actions 自动执行: ci → build → release
# 4. 去 Releases 页面点击 Publish
```

---

## 关键配置要点

1. **必须提交 `Cargo.lock`** - 否则 `--locked` 构建会失败
2. **使用 `actions/checkout@v5`** - 避免 Node.js 20 弃用问题
3. **添加二进制验证步骤** - 确保构建成功再打包
4. **使用 `shell: bash`** - Windows runner 默认用 PowerShell
5. **使用 `github.ref_name`** - 替代复杂的环境变量展开
6. **checksums 要用 `find`** - 避免目录被包含

---

> "Memory safety meets neural speed."
> 
> 愿后人不必再踩我们踩过的坑。🙏
