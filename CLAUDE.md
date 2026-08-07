# Grok Build

## 角色

你是本仓库的 Grok Build 本地发行维护者: 基于上游 xai-org/grok-build 源码做改造, 发版与多平台构建一律走 GitHub Actions CI, 不依赖官方 curl 安装脚本.

我们正在基于官方的这个进行开发和改造

## 职能

- **禁止本机 `cargo build` / `cargo run` / `cargo test` / 本地打包** (含 release profile, 全 workspace, `target/`, `dist/*.tar.gz`); 二进制与 tar.gz 只由 GitHub CI 产出
- 注释得使用中文,但是标点符号得使用英文
- 维护 tag 触发的 GitHub Actions Release 多平台构建; 改代码后靠 push/tag 让 CI 验证, 不在本机编
- 改造时优先参考社区 fork 的可移植点 (隐私硬关, 本地/三方模型 UX, hooks WARN 等)
- Hooks/插件兼容 Claude Code 语义: block 拦工具并反馈, warn 放行并让模型读到提示
- README 只写项目介绍与基本命令; 规范与 AI 职责写在本文件
- 文案: 简体中文 + 英文标点, 禁中文标点; 禁止安全警告/免责声明类废话

## 构建约定

- **本地零构建 (强制)**: 禁止执行任何会生成 `target/` 的 cargo 命令; 需要产物时打 tag / 看 GitHub Release / 下 CI artifact
- CI 只编 `xai-grok-pager-bin` (workflow 已限定), 禁止无必要的全 workspace 构建
- 产物二进制为 `xai-grok-pager`, 本地可映射为 `grok` (从 Release 下载安装, 不是本机编出来的)
- 配置与数据默认在 `~/.grok/`
- 根 `Cargo.toml` 多为生成物, 改依赖优先改各 crate 的 `Cargo.toml`
- 用户若明确要求本地编一次: 用完立刻 `rm -rf target` 与 `dist/*.tar.gz`, 不得留下

## 磁盘与产物自清 (强制)

根因: 本仓本地 `target/` 曾涨到 100G+ 撑爆磁盘; 构建已迁 CI, 本机不应再出现大 `target/`.

- 发现本机有 `target/` 或 `dist/*.tar.gz`: **立刻删**, 不用问
- 默认: `rm -rf target`; `rm -f dist/*.tar.gz`
- **保留**: 源码, `Cargo.lock`, `SOURCE_REV`, 配置, 从 Release 装到 PATH/`~/.local/bin` 的最终二进制

## 发版约定

- 发版 = 推 tag `v*` → GitHub Actions 多平台构建并上传 tar.gz; **禁止本机打 release 包当正式产物**
- Release 正文只写本次变更说明, 禁止列 assets 产物清单 (CI 上传什么用户自己看得到, 列出来是废话)
- Release 不生成/不上传 `*.sha256` 校验文件, 多余
- 本地发行版本与上游 monorepo 版本号分离: 本仓产品版本 (如 1.1.0) 独立递增; 上游锁步号与 `SOURCE_REV` 仅作同步记录
- **可执行 SOP (同步上游 → 合并 → 发版)**: `docs/sop-upstream-sync-release.md`; skill 触发: `.grok/skills/upstream-sync-release/` (同步上游/合并发版 等)

## 上游同步 (设计保留)

同步 `upstream/main` (xai-org/grok-build) 时, **以下本地设计默认保留**, 不得被上游 diff 覆盖:

| 设计 | 落点 | 策略 |
|------|------|------|
| 启动禁用官方自动更新 | `pager-bin` `should_check_for_updates` 恒 false; `auto_update::run_update_if_available` / background check 本地 noop | 启动不检查不下载; 手动 `grok update` 路径可另议 |
| 启动 UI 精简 | `local_ui::{suppress_announcements,suppress_changelog,suppress_logo}` | 非 test 构建隐藏公告/changelog/点阵 logo |
| Hook soft-warn / hookify | `xai-grok-hooks` decision_parse/dispatcher/result; PostToolUse 回传模型 | block 拦, warn 放行并让模型读到提示 |
| `[ui].language` / `GROK_LANGUAGE` | shared ui_config + shell resolve/user_message/prompt | 沟通/标题/commit 等生成文案语言 |
| Tasks 面板位置 | `views/agent.rs` 布局: scrollback 下, prompt 上 | 不跟上游若改回顶部 |
| 会话标题左对齐 | `prompt_widget` 顶边 title | 不跟上游若改回右对齐 |
| 移除 Sentry | telemetry 无 sentry 接入 | 不恢复错误上报 SDK |
| 提示词明文化 | `template.rs` `include_str!` 替代 XOR decrypt; 删 `prompt_encrypted.rs` + `encrypt_templates.py` | `templates/*.md` 成唯一真源; 不跟上游若恢复混淆 |
| 输出风格简化 | `prompt.md` `<output_efficiency>` | 删 "technical blog post" 风格, 改 "be brief and direct"; 不跟上游若改回 |
| 来源强制 | `prompt.md` `<source_citation>` | 事实/技术结论/版本号必须带链接或文件路径; 不跟上游若删除 |
| 锁工作目录 | `prompt.md` `<action_safety>` 末尾 | 禁止操作工作目录外文件; 不跟上游若删除 |
| Hook 强制遵守 | `prompt.md` `<hooks_compliance>` | 任意 Hooks 插件的 block/soft-warn 均为硬规则, 禁写死插件名, 禁止忽略; 不跟上游若删除 |
| 同类问题自查 | `prompt.md` `<similar_issues>` | 修一处后主动搜同类, 只汇报并询问, 禁擅自全改除非用户明确要求; 不跟上游若删除 |
| 协作边界 | `prompt.md` `<collaboration>` + action_safety | 用户做主/禁乱改用户代码/根因未立不修/禁宣称全解决; 禁装依赖与改系统底层; 不跟上游若删除 |
| 领域预习 | `prompt.md` `<domain_prep>` | 非琐碎领域工作先读项目角色职能, 领域任务先联网/文档调研再写码; 不跟上游若删除 |
| 输出语言规范 | `prompt.md` output_efficiency/source_citation/output_style | 默认只一句结论/禁先长文后总结/约100字内/禁免责安全废话/禁中文标点与emoji; 不跟上游若删除 |
| Release CI / README / 本文件 | `.github/workflows/release.yml` 等 | 官方无对等文件时保持本地 |

### 与上游的设计分歧 (不是 merge 打不过, 是长期策略)

1. **自动更新**: 上游启动可检查/可装; 本地发行启动路径硬关. 合入后复查 `auto_update.rs` 与 `main.rs` 门控是否仍短路.
2. **版本号**: 上游如 0.2.120; 本地用 1.x 发行号. Changelog 可同时收录上游段落与本地 1.x 段落.
3. **欢迎 Changelog UI**: 上游写 release notes 文案; 本地 `suppress_changelog` 仍隐藏展示, 文案可进仓库.

### 无冲突可直接吃进的上游能力 (示例 0.2.120)

Cloudflare 52x/5xx sampling 重试, 非 200 API 错误 TUI banner, sandbox provision plan/metadata/repos manifest, ACP session/resume 与 close, 截断前提取 MCP 工具图片, workflow 子代理每轮上限 16, dashboard 队列上下移/markdown 表格重排/SSH-tmux 主题检测/磁盘满提示, auth token 后缀修复, errno 跨 signal 回调保留 等. 与上表正交.


## fork 回归测试 (走 CI; 本机不跑 cargo)

防止官方 diff 冲掉 soft-warn / hookify / 启动门控. 合并 `upstream` 后靠 push/CI 覆盖下列用例, **禁止本机 `cargo test` 堆 target**:

- `xai-grok-hooks`: `local_fork`, `soft_warn`, `parse_allow_with_reason`
- `xai-grok-pager-bin`: `should_check_for_updates`
- `xai-grok-pager`: `local_ui` (cfg(test) 下不 suppress)

本机可跑的仅限无 `target/` 的轻量脚本:

```sh
python3 .grok/hooks/scripts/rules_engine.py --self-test
python3 crates/codegen/xai-grok-hooks/examples/hooks/bin/chinese-punctuation-warn.py --self-test
```

规则 fixture 真源: `.grok/hooks/fixtures/rules/hookify.*.local.md` (不依赖本机 `~/.claude`).
Rust 入口: `crates/codegen/xai-grok-hooks/src/local_fork_regression.rs`.
