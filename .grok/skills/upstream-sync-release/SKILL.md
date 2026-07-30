---
name: upstream-sync-release
description: 当用户要求 "同步上游", "合并上游", "拉上游发版", "合并线上最新", "合并主分支上游", "同步并发布", "上游合并发版", 或需要按本仓流程把 xai-org/grok-build 合进 main 并打 tag 发 GitHub Release 时使用. 完整步骤见 docs/sop-upstream-sync-release.md.
---

# 上游同步 + 发版 (本仓)

## 必读

执行前完整阅读并按步骤跑:

**`docs/sop-upstream-sync-release.md`**

不要只按全局 `release-workflow` 打 tag; 本仓强制:

1. `git fetch upstream` 并 merge `upstream/main`
2. 保留 `Claude.md` 本地设计表 (关更新 / local_ui / soft-warn / language / 无 Sentry / CI)
3. 产品版本 1.x 独立递增; 上游 0.2.x 只记 SOURCE_REV
4. **禁止本机 cargo** 与本地 tar.gz 正式产物
5. `gh release create` 必须 `--repo phpmac/grok-build`

## 最短执行序

```
fetch → (count HEAD..upstream/main?) → branch sync/upstream-*-1.x.y
→ merge → 门控复查 → 升 1.x + CHANGELOG + Claude.md
→ python 自检 → commit → ff main → push → tag v* → gh release
→ 看 CI assets
```

## 触发后行为

用户说"同步上游 / 合并发版"等 → 直接按 SOP 全流程执行, 不必再问"是否发版"(已隐含).  
仅当工作区有未说明的脏改动时, 先问清 commit 还是 stash.
