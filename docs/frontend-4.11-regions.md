# 4.11 区域加载与失败恢复子项

2026-10-28，已部署正式前端。4.11/P5总体仍未勾选。

## 改动文件
- src/components/RegionState.svelte：复用静态骨架、区域错误及重试，支持紧凑布局和可访问状态，不新增依赖。
- src/studio.svelte.js：初始化完成前不宣称ready，明确startupState/startupError，可重试恢复。
- src/components/CompleteStudio.svelte：初始化骨架/错误行；未就绪时标题/工具栏inert，暂不显示对话和编辑面板。
- src/services/task-monitor.js：首次读取、失败和重试状态，保留已有任务；旧工作区错误不能污染新工作区。
- src/components/TaskCenter.svelte：任务骨架与有效重试，读取失败不显示空任务。
- src/components/HomePage.svelte：最近任务独立加载/错误/重试，不将读取失败当作空列表。
- src/stores/chat.svelte.js：任务同步失败归到任务区域，不显示成指令失败。
- src/components/AssetTile.svelte：单图缩略图失败可重试，版本变化清旧图；重试不触发框选/选中。
- src/components/Settings.svelte：读取设置骨架、错误和重试。
- src/components/Export.svelte：读取导出效果骨架和原参数重试。
- scripts/test-region-states.mjs：4项新增回归，覆盖初始化未完成/失败/恢复、任务状态、旧工作区失败响应隔离，以及可选缩略图失败不阻塞工作区。
- scripts/verify-region-states.mjs：发布包真实浏览器，注入读取延迟和503后点击重试。
- docs/frontend-decisions.md：D057/D058；frontend-4.11-audit.md：同步剩余范围。

## 验收
✓ 工作区未完成初始化时显示骨架，不露出可提交的对话/编辑面板；失败可重试到正常。
✓ 任务读取失败不显示“暂无任务”；重试骨架后才显示真实空列表，成功清除错误。
✓ 单张缩略图失败可重试，恢复图像且未误选素材。
✓ 无匹配素材可清空筛选回到网格。
✓ 设置、导出效果从加载到失败到重试成功；导出无需改参数触发重试。
✓ 原有编辑器草稿撤销、自然语言一步执行、新品滚动/分配/澄清/ZIP回归通过。
✓ coverflow冻结源码检查通过，Svelte检查和生产构建通过。
✓ 正式index哈希与构建一致，一键启动OpenCV4.8.0可用，4项目/63素材ID、原图SHA、版本数保持。

## 测试证据
前端纯测试73/73、浏览器条件回归30/30，共103项。故障注入UI验证6组，另有编辑器/管线/新品17组回归。测试使用隔离API夹具，付费调用0，无真实项目写操作。
output/region-red.log、region-task-red.log记录原问题；region-tests.log、region-pure-tests.log记录绿灯结果。浏览器证据output/playwright/region-states/receipt.json、workspace-loading.png、workspace-error.png、thumbnail-error.png；部署证据deployment.json。

正式入口http://127.0.0.1:8787/，继续使用一键启动.cmd。本次只更新静态前端、保留旧哈希资源；旧index备份.tools/deployment-backups/20261028-070856-p4-regions/。

未完成的全量范围：普通生图/自然语言配置读取等其余区域、旧成功toast/弹窗入口清理、完整mock入口、所有T性能指标，以及P5外部目录/团队并发等。不得用本子项证据勾选整个4.11/P5。
