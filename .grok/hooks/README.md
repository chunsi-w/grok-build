# hookify 规则资产 (回归自检用)

Claude Code hookify 插件兼容. 运行时不注册任何本地 JSON hook, 规则执行完全走
Claude Code 生态的 hookify 插件 (Grok 经 plugins.paths 加载, 同一插件两侧通用).

## 运行时链路 (Claude Code 兼容)

- 规则真源: `~/.claude/hookify.*.local.md` 与 `<项目>/.claude/hookify.*.local.md`
- 执行者: hookify 插件 `hooks/hooks.json` (PreToolUse/PostToolUse/Stop/UserPromptSubmit/SubagentStart)
- 插件启用即在会话自动注册, 无需任何手工软链或 mv
- 改规则文件即时生效; 插件自身改动需新开会话

## 本目录资产

- `scripts/rules_engine.py`: fork 回归引擎, 只用于自检与 Rust 回归测试, 不注册运行时
- `fixtures/rules/`: 固定规则副本, 供自检使用, 不依赖 `~/.claude`

## 回归自测

```sh
# 规则引擎: 中文标点 soft-warn, github 禁爬虫/curl, gh 放行
# 装机自检: ~/.claude 存在时, 中文标点规则必须在位
python3 .grok/hooks/scripts/rules_engine.py --self-test

# 也可经 cargo 锁协议 + 脚本 (合并上游后必跑)
cargo test -p xai-grok-hooks local_fork
```
