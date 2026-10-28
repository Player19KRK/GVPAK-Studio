# 4.11 编辑器撤销与面板入口

2026-10-28，本子项完成并部署正式前端；4.11/P5整体继续保持未勾选。

## 文件
- src/services/commands.js：编辑器优先本机草稿撤销/重做，有无关草稿时不提交旧批次重做；素材库保留批次作用域。
- src/studio.svelte.js：恢复草稿无内存历史时可撤销到保存编辑；Svelte数组初始化后从代理读取再修改，使首次重做可追踪。
- src/stores/chat.svelte.js：undo/redo与commit_edit一样不经过草稿自动提交，避免改变批次前置条件。
- src/components/Inspector.svelte：恢复草稿时撤销按钮可用。
- src/components/CompleteStudio.svelte：调色工具按点击前可见状态判定开关，对话展开时首次点击即可显示面板；aria-expanded对应实际可见面板。
- scripts/test-history-priority.mjs：6项回归，真实命令+状态模块、实际sendChat链路；修复前3项优先级测试及自动保存测试失败，修复后全通过。
- scripts/verify-history-priority.mjs：开发页面实际Ctrl+Z、Ctrl+Shift+Z与执行请求检查。
- scripts/verify-history-production.mjs：发布包实际曝光控件、页面刷新后的草稿、按钮、批次衔接和文本原生撤销。所有API截获为隔离夹具，无付费请求、不修改正式素材。
- docs/frontend-decisions.md：D055决策；frontend-4.11-audit.md：更新证据。

## 验收
✓ 当前草稿撤销/重做优先于已有批次，且不提交图片处理。
✓ 实际刷新后恢复草稿，撤销按钮可用，撤销/重做值正确。
✓ 清洁编辑器继续撤销时，仅提交一次原批次操作，无edit_commit。
✓ 素材库维持批次作用域，文本输入框维持原生撤销。
✓ 调色入口首次打开正常，无浏览器页面异常。
✓ 相关26项单元回归通过、Svelte检查和Vite发布构建通过，coverflow冻结检查通过。

证据：output/history-red.log、history-red-save.log、history-tests.log；output/playwright/history/receipt.json、production-receipt.json。
正式前端已更新到http://127.0.0.1:8787/，旧入口备份在.tools/deployment-backups/20261028-064823-p4-history/。无需重启后端，无后端逻辑或依赖变更；一键启动检查OpenCV4.8.0可用。

剩余范围：区域四状态、完整主题与性能、外部目录和跨设备队列等，详见frontend-4.11-audit.md。
