# 4.11 配置读取状态子项

2026-10-28，已进入正式完整应用。4.11及P5整体仍未勾选。

## 改动文件
- src/lib/configuration.svelte.js：只读配置的idle/loading/ready/error、未配置false、重试及过期响应隔离；离开入口仅停止采用旧响应，不宣称取消网络请求。
- src/components/Generate.svelte：普通生图显示读取骨架、错误重试、空模型/未配置提示；Google/Poe按实际选择区分；切换时立即撤销旧可提交状态，按需读取。
- src/components/Assistant.svelte：自然语言处理配置读取与错误重试，去除重复读取；本地步骤继续独立可用，同事端不跳转受限设置。
- scripts/test-configuration.mjs：3项状态、失败恢复和模型切换竞态单元测试。
- scripts/verify-configuration-states.mjs：生产构建的Google503/延迟、Poe未配置、旧Google响应及DeepSeek恢复；使用当前模型目录选择Poe。
- scripts/studio-test-harness.mjs：接入真实配置状态模型，保留原编辑/生图回归。
- docs/frontend-decisions.md：D061；frontend-4.11-audit.md：更新剩余范围。

## 验收
✓ 读取中不再误报未配置，失败明确显示错误和原地重试。
✓ 已成功读到false才显示未配置；Google/Poe提示对应当前模型。
✓ 切换至未配置的Poe后，迟到Google=true不会启用生成按钮。
✓ DeepSeek读取失败后重试恢复自然语言处理按钮；不提交任何处理任务。
✓ 同事端未配置提示联系主机，主机端保留设置入口。
✓ 确认门槛的宿主提示改为同一执行行更新，避免任务完成后仍指示等待确认。
✓ 普通生图6组、自然语言5组、新品7组浏览器回归，无页面异常。
✓ coverflow冻结检查通过，无后端或依赖变更。

## 测试与证据
纯前端79/79、浏览器条件34/34，共113项；Svelte检查与发布构建通过。配置浏览器专项5组，付费调用0。首次故障注入在旧版读取加载断言失败，随后新实现通过；状态单测覆盖false与错误区别、读新模型清旧enabled、迟到响应/离开入口不污染当前状态。
证据：output/configuration-tests.log、configuration-pure-tests.log；output/playwright/configuration-states/receipt.json、generation-error.png、deployment.json。

正式http://127.0.0.1:8787/与构建哈希一致，一键启动OpenCV4.8.0可用，4项目/63素材ID、原图SHA、版本数保持。本次仅更新静态前端，未重启后台；旧入口备份.tools/deployment-backups/20261028-074019-p4-configuration/。

旧成功toast/Confirm/Review入口、完整mock、其他区域状态及剩余P5继续处理，不用此子项证据替代整体交付。
