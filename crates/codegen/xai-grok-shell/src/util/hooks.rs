//! Host bindings for [`xai_grok_hooks::discovery`]. Discovery itself lives in the hooks crate.
//! Bound here are the two inputs that crate cannot read: the Claude import cutoff and the managed-settings hooks pin.

use std::path::Path;

use xai_grok_hooks::error::HookError;

/// The `[claude_compat] imported = true` cutoff, read once per process in [`crate::claude_import`].
/// Every entry point below passes it down so the hooks crate stays free of shell state.
fn claude_import_marked() -> bool {
    crate::claude_import::is_claude_import_marked_with_log("discover_hook_source_paths")
}

/// The disabled-hooks file plus the resolved `allow_managed_hooks_only` pin.
pub(crate) fn disabled_hooks_snapshot() -> xai_grok_hooks::trust::DisabledHooks {
    let managed_only = xai_grok_workspace::permission::resolution::managed_settings()
        .non_managed_hooks
        .is_disabled();
    xai_grok_hooks::trust::DisabledHooks::load(managed_only)
}

/// Single load entry point: [`xai_grok_hooks::discovery::discover_hooks`] with the Claude cutoff applied.
/// Every session-startup and mid-session reload site routes through here so the source policy stays in one place.
pub(crate) fn discover_hooks(
    git_root: Option<&Path>,
    compat: &xai_grok_tools::types::compat::CompatConfig,
    trusted: bool,
) -> (xai_grok_hooks::discovery::HookRegistry, Vec<HookError>) {
    xai_grok_hooks::discovery::discover_hooks(git_root, compat, claude_import_marked(), trusted)
}

/// [`xai_grok_hooks::discovery::assemble_hooks`] with the Claude cutoff applied, for callers that
/// supply their own config layers.
pub(crate) fn assemble_hooks(
    config_layers: &[xai_grok_config::HookConfigLayer],
    git_root: Option<&Path>,
    compat: &xai_grok_tools::types::compat::CompatConfig,
    trusted: bool,
) -> (xai_grok_hooks::discovery::HookRegistry, Vec<HookError>) {
    xai_grok_hooks::discovery::assemble_hooks(
        config_layers,
        git_root,
        compat,
        claude_import_marked(),
        trusted,
    )
}

/// 收集插件贡献的 hook specs (文件 hooks.json + manifest inline hooks).
///
/// 会话启动路径与 reload 路径共用: 此前只有 reload 路径
/// (`apply_plugin_registry_snapshot`) 挂插件 hooks, 冷启动会话的 hook
/// 注册表恒不含它们, 插件拦截静默失效.
/// specs 名带 `plugin/` 前缀, reload 时按前缀整体移除后重挂, 不产生重复.
pub(crate) fn collect_plugin_hook_specs(
    plugins: &[&xai_grok_agent::plugins::LoadedPlugin],
) -> Vec<xai_grok_hooks::config::HookSpec> {
    let mut specs = Vec::new();
    for plugin in plugins {
        if let Some(ref hooks_path) = plugin.hooks_path {
            let (mut parsed, warnings) =
                xai_grok_agent::plugins::hooks_adapter::parse_plugin_hooks(
                    hooks_path,
                    &plugin.name,
                    &plugin.root_str(),
                    &plugin.data_dir_str(),
                );
            for w in warnings {
                tracing::warn!("{w}");
            }
            specs.append(&mut parsed);
        }
        if let Some(ref inline_value) = plugin.inline_hooks {
            let (mut parsed, warnings) =
                xai_grok_agent::plugins::hooks_adapter::parse_plugin_hooks_from_value(
                    inline_value,
                    &plugin.name,
                    &plugin.root_str(),
                    &plugin.data_dir_str(),
                );
            for w in warnings {
                tracing::warn!("{w}");
            }
            specs.append(&mut parsed);
        }
    }
    specs
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_hooks::config::HookProvenance;
    use xai_grok_hooks::error::HookError;
    use xai_grok_hooks::event::HookEventName;

    /// 构造一个 LoadedPlugin 测试夹具, hooks 相关字段由调用方指定.
    fn loaded_plugin_fixture(
        name: &str,
        hooks_path: Option<std::path::PathBuf>,
        inline_hooks: Option<serde_json::Value>,
    ) -> xai_grok_agent::plugins::LoadedPlugin {
        use xai_grok_agent::plugins::discovery::PluginId;
        use xai_grok_agent::plugins::{PluginOrigin, PluginScope};
        let root = std::path::PathBuf::from(format!("/tmp/test-plugins/{name}"));
        xai_grok_agent::plugins::LoadedPlugin {
            name: name.to_string(),
            id: PluginId::new(PluginScope::User, &root, name),
            root: root.clone(),
            canonical_root: root.clone(),
            scope: PluginScope::User,
            origin: PluginOrigin::UserGrok,
            trusted: true,
            enabled: true,
            version: Some("1.0.0".to_string()),
            description: Some(format!("test plugin {name}")),
            skill_dirs: vec![],
            command_dirs: vec![],
            agent_dirs: vec![],
            hooks_path,
            mcp_config_path: None,
            lsp_config_path: None,
            skill_count: 0,
            agent_count: 0,
            skill_names: vec![],
            agent_names: vec![],
            has_hooks: hooks_path.is_some() || inline_hooks.is_some(),
            hook_count: 0,
            has_inline_hooks_only: hooks_path.is_none() && inline_hooks.is_some(),
            mcp_server_count: 0,
            has_inline_mcp_only: false,
            lsp_server_count: 0,
            has_inline_lsp_only: false,
            inline_hooks,
            inline_mcp_servers: None,
            inline_lsp_servers: None,
            conflict: None,
        }
    }

    /// 启动路径回归: 插件文件 hooks 必须被收集, 名带 `plugin/` 前缀
    /// (reload 的 remove_by_prefix 依赖该前缀去重), 且能装入注册表.
    #[test]
    fn plugin_file_hooks_are_collected_for_startup_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let hooks_dir = tmp.path().join("hooks");
        std::fs::create_dir_all(&hooks_dir).unwrap();
        std::fs::write(
            hooks_dir.join("hooks.json"),
            r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"python3 pre.py"}]}]}}"#,
        )
        .unwrap();

        let plugin = loaded_plugin_fixture(
            "hookify",
            Some(hooks_dir.join("hooks.json")),
            None,
        );
        let specs = collect_plugin_hook_specs(&[&plugin]);
        assert_eq!(specs.len(), 1, "one PreToolUse spec expected");
        assert!(
            specs[0].name.starts_with("plugin/hookify/"),
            "spec name must carry the plugin/ prefix for reload dedup, got {}",
            specs[0].name
        );
        assert_eq!(specs[0].event, HookEventName::PreToolUse);

        // 装入注册表后 PreToolUse 可见 (spawn 启动路径的等效断言).
        let mut registry = xai_grok_hooks::discovery::HookRegistry::default();
        registry.append_specs(specs);
        assert_eq!(
            registry.hooks_for(HookEventName::PreToolUse).len(),
            1,
            "startup registry must expose the plugin PreToolUse hook"
        );
    }

    /// 无 hooks 的插件不得贡献任何 spec (对其他插件零影响).
    #[test]
    fn plugin_without_hooks_contributes_no_specs() {
        let plugin = loaded_plugin_fixture("no-hooks", None, None);
        let specs = collect_plugin_hook_specs(&[&plugin]);
        assert!(specs.is_empty(), "expected no specs, got {specs:?}");
    }

    /// manifest inline hooks 与文件 hooks 一样被收集.
    #[test]
    fn plugin_inline_hooks_are_collected() {
        let inline = serde_json::json!({
            "hooks": {
                "PreToolUse": [{"hooks": [{"type": "command", "command": "inline.sh"}]}]
            }
        });
        let plugin = loaded_plugin_fixture("inline-only", None, Some(inline));
        let specs = collect_plugin_hook_specs(&[&plugin]);
        assert_eq!(specs.len(), 1, "inline PreToolUse spec expected");
        assert_eq!(specs[0].event, HookEventName::PreToolUse);
    }

    /// 多插件混合 (带 hooks / 不带 hooks) 只收集带 hooks 的; reload 语义下
    /// remove_by_prefix("plugin/") 再重挂不会累积重复.
    #[test]
    fn mixed_plugins_collect_only_hooked_ones_and_reload_dedups() {
        let tmp = tempfile::tempdir().unwrap();
        let hooks_dir = tmp.path().join("hooks");
        std::fs::create_dir_all(&hooks_dir).unwrap();
        std::fs::write(
            hooks_dir.join("hooks.json"),
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"stop.py"}]}]}}"#,
        )
        .unwrap();

        let hooked = loaded_plugin_fixture("hookify", Some(hooks_dir.join("hooks.json")), None);
        let plain = loaded_plugin_fixture("notify", None, None);
        let specs = collect_plugin_hook_specs(&[&hooked, &plain]);
        assert_eq!(specs.len(), 1, "only the hooked plugin contributes");

        let mut registry = xai_grok_hooks::discovery::HookRegistry::default();
        registry.append_specs(specs.clone());
        // 模拟 reload: 先按前缀整体移除再重挂.
        registry.remove_by_prefix("plugin/");
        registry.append_specs(specs);
        assert_eq!(
            registry.hooks_for(HookEventName::Stop).len(),
            1,
            "re-appending after prefix removal must not duplicate hooks"
        );
    }

    /// Write `content` as `<dir>/requirements.toml`.
    fn write_requirements(dir: &Path, content: &str) {
        std::fs::write(dir.join("requirements.toml"), content).unwrap();
    }

    /// A temp policy layer pins hooks for `SessionStart`, `UserPromptSubmit`, and `PreToolUse`.
    /// It flows through the real requirements read (`hook_config_layers_at`) and the real assembly (`assemble_hooks`).
    /// All three register with `Requirements` provenance, the provenance the disable exemption keys on.
    #[test]
    fn requirements_layer_pins_hooks_with_requirements_provenance() {
        let system_dir = tempfile::tempdir().unwrap();
        write_requirements(
            system_dir.path(),
            r#"
[[hooks.SessionStart]]
[[hooks.SessionStart.hooks]]
type = "command"
command = "/opt/policy/pin-session-start.sh"
timeout = 5

[[hooks.UserPromptSubmit]]
[[hooks.UserPromptSubmit.hooks]]
type = "command"
command = "/opt/policy/pin-prompt-submit.sh"
timeout = 5

[[hooks.PreToolUse]]
matcher = "*"
[[hooks.PreToolUse.hooks]]
type = "command"
command = "/opt/policy/pin-pre-tool-use.sh"
timeout = 5
"#,
        );

        let layers = xai_grok_config::hook_config_layers_at(Some(system_dir.path()), None);
        assert_eq!(layers.len(), 1, "one requirements layer expected");
        let Some(layer) = layers.first() else {
            panic!("one requirements layer expected: {layers:?}");
        };
        assert_eq!(layer.provenance(), HookProvenance::Requirements);
        assert_eq!(layer.source_name(), "requirements/system");

        let compat = xai_grok_tools::types::compat::CompatConfig::default();
        let (registry, errors) = assemble_hooks(&layers, None, &compat, false);
        assert!(errors.is_empty(), "errors: {errors:?}");

        for (event, command) in [
            (HookEventName::SessionStart, "pin-session-start.sh"),
            (HookEventName::UserPromptSubmit, "pin-prompt-submit.sh"),
            (HookEventName::PreToolUse, "pin-pre-tool-use.sh"),
        ] {
            let spec = registry
                .hooks_for(event)
                .iter()
                .find(|s| {
                    s.command_raw
                        .as_deref()
                        .is_some_and(|c| c.contains(command))
                })
                .unwrap_or_else(|| panic!("pinned {event} hook must register"));
            assert_eq!(
                spec.layer,
                HookProvenance::Requirements,
                "pinned {event} hook must carry requirements provenance"
            );
            assert!(
                spec.is_managed_policy(),
                "requirements provenance must classify as managed policy"
            );
            assert!(
                spec.name.starts_with("requirements/system:"),
                "provenance-prefixed name expected, got {}",
                spec.name
            );
        }
    }

    /// A realistic enterprise policy hooks shape parses and registers through the real path.
    /// The shape: command hooks with `timeout: 5`, `PreToolUse` with `matcher: "*"` and two hooks in one group, and matcher-less lifecycle groups.
    /// The two `PreToolUse` hooks are byte-identical, so both parse but content dedup registers one effective hook.
    #[test]
    fn enterprise_policy_hooks_shape_registers() {
        let system_dir = tempfile::tempdir().unwrap();
        write_requirements(
            system_dir.path(),
            r#"
[[hooks.SessionStart]]
[[hooks.SessionStart.hooks]]
type = "command"
command = "policy/hooks/bin/lifecycle-audit.sh"
timeout = 5

[[hooks.PreToolUse]]
matcher = "*"
[[hooks.PreToolUse.hooks]]
type = "command"
command = "policy/hooks/bin/pretooluse-audit.sh"
timeout = 5
[[hooks.PreToolUse.hooks]]
type = "command"
command = "policy/hooks/bin/pretooluse-audit.sh"
timeout = 5

[[hooks.UserPromptSubmit]]
[[hooks.UserPromptSubmit.hooks]]
type = "command"
command = "policy/hooks/bin/lifecycle-audit.sh"
timeout = 5
"#,
        );

        let layers = xai_grok_config::hook_config_layers_at(Some(system_dir.path()), None);
        assert_eq!(layers.len(), 1);

        // Parse level: the verbatim structure yields both PreToolUse handlers.
        let (specs, errors) = xai_grok_hooks::config::parse_hooks_from_config_layers(&layers);
        assert!(errors.is_empty(), "errors: {errors:?}");
        let pre_specs: Vec<_> = specs
            .iter()
            .filter(|s| s.event == HookEventName::PreToolUse)
            .collect();
        assert_eq!(
            pre_specs.len(),
            2,
            "the PreToolUse group's two hooks must both parse"
        );
        for spec in &pre_specs {
            assert_eq!(spec.configured_matcher.as_deref(), Some("*"));
            let matcher = spec.matcher.as_ref().expect("matcher '*' compiles");
            assert!(
                matcher.is_match("run_terminal_command") && matcher.is_match("Bash"),
                "matcher '*' must match every tool"
            );
            assert_eq!(spec.timeout_ms, 5000, "timeout 5s converts to 5000ms");
        }

        // Registry level through the real assembly: all three events register with requirements provenance
        // The byte-identical PreToolUse duplicate collapses to one effective hook
        let compat = xai_grok_tools::types::compat::CompatConfig::default();
        let (registry, errors) = assemble_hooks(&layers, None, &compat, false);
        assert!(errors.is_empty(), "errors: {errors:?}");
        for event in [
            HookEventName::SessionStart,
            HookEventName::UserPromptSubmit,
            HookEventName::PreToolUse,
        ] {
            let policy_hooks: Vec<_> = registry
                .hooks_for(event)
                .iter()
                .filter(|s| s.layer == HookProvenance::Requirements)
                .collect();
            assert!(
                !policy_hooks.is_empty(),
                "pinned {event} hook must register with requirements provenance"
            );
        }
        assert_eq!(
            registry
                .hooks_for(HookEventName::PreToolUse)
                .iter()
                .filter(|s| s.layer == HookProvenance::Requirements)
                .count(),
            1,
            "byte-identical duplicate collapses under content dedup"
        );
    }

    #[test]
    fn directory_at_cursor_hooks_json_does_not_load_child_hooks() {
        let root = tempfile::tempdir().unwrap();
        let disguised = root.path().join(".cursor").join("hooks.json");
        std::fs::create_dir_all(&disguised).unwrap();
        std::fs::write(
            disguised.join("startup.json"),
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"cursor_hooks_json_dir_probe.sh"}]}]}}"#,
        )
        .unwrap();

        let compat = xai_grok_tools::types::compat::CompatConfig::default();
        let (registry, errors) =
            assemble_hooks(&[], Some(root.path()), &compat, /*trusted*/ true);

        assert!(
            !registry.all_hooks().iter().any(|h| {
                h.command_raw
                    .as_deref()
                    .is_some_and(|c| c.contains("cursor_hooks_json_dir_probe"))
            }),
            "child JSON under a directory at .cursor/hooks.json must not load as hooks"
        );
        assert!(
            errors.iter().any(|e| matches!(
                e,
                HookError::ReadFile { path, .. } if path == &disguised
            )),
            "reading the disguised directory as a settings file must surface ReadFile; got {errors:?}"
        );
    }
}
