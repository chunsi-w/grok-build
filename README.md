# Grok Build

终端 AI 编程 Agent, 本地编译运行.

## 使用

```sh
# 编译并启动
cargo run -p xai-grok-pager-bin

# 发布构建
cargo build -p xai-grok-pager-bin --release
./target/release/xai-grok-pager

# 可选
cp target/release/xai-grok-pager ~/.local/bin/grok
grok
```

Mac 上若 SIGKILL: `xattr -cr ~/.local/bin/grok`

## 配置

`~/.grok/config.toml` 常用项:

```toml
[ui]
language = "简体中文"    # 沟通/标题/commit 等生成文案语言; 也可设 GROK_LANGUAGE
```

启动时只显示版本, 不做官方自动更新检查.

## 发版

本机打包, 不经 CI:

```sh
cargo build --release --locked -p xai-grok-pager-bin --target aarch64-apple-darwin

dist=dist/grok-aarch64-apple-darwin
mkdir -p "$dist"
cp target/aarch64-apple-darwin/release/xai-grok-pager "$dist/grok"
chmod +x "$dist/grok"
tar -czf "${dist}.tar.gz" -C dist grok-aarch64-apple-darwin
```

打 tag 并把本地产物挂到 Release (tag 本身不再触发构建):

```sh
git tag v1.24.0 && git push origin v1.24.0
gh release create v1.24.0 --repo phpmac/grok-build --target main \
  --title "<一句话中文主题>" --notes-file <变更说明> \
  dist/grok-aarch64-apple-darwin.tar.gz
```

