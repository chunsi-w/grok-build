---
name: block-php-type-cast-pre
enabled: true
event: file
action: block
tool_matcher: Write|Edit
hook_events:
  - PreToolUse
file_matcher: \.php$
conditions:
  - field: content
    operator: regex_match
    pattern: '\((int|integer|string|float|double|bool|boolean|array|object)\)\s*[$''"]'
---

**PHP 强制类型转换, 检查是否违反规范**

写入被拦截 (Post 阶段 sg 规则 `warn-php-type-cast` 仍会回传行号兜底)

- `(int) $x` / `(string) $this->name` → Laravel Model 对象字段通常用 `$casts` 自动转换, 无需强转
- `(int)`/`(string)`/`(float)`/`(bool)`/`(array)`/`(object)` 后跟变量/字符串即命中; 注释或字符串内文本可能误报, 由 `warn-php-type-cast` 的 sg 精准扫描二次确认
