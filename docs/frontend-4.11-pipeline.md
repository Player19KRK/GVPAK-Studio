# 4.11 自然语言混合管线交付记录

2026-10-28。本子项已进入正式一键启动版本，4.11及P5整体仍未勾选，剩余项见frontend-4.11-audit.md。

## 行为
输入要求后直接开始，默认实时，保留异步Batch和既有模型。费用、数量、配额门槛在对话执行行继续同一请求。自然语言规划使用DeepSeek非思考流式工具调用，素材上下文只有8字段SelectionContext，fileNames为文件名。实时处理转成标准任务，Batch复用旧引擎并隐藏内部重复任务。

纯格式/压缩导出保留实际尺寸；纯导出不新增图片版本且无图片撤销按钮。仅指定比例时保留长边补边。返回图片与交付文件分开校验；保留新编辑，取消后接回已返回像素，不自动重发未知请求。新品识别原始参考不随生成版本改变。

## 改动文件
- backend/src/frontend_pipeline.rs：管线工具schema、规划、校验与现有操作转换。
- backend/src/frontend_chat.rs：复用流式单工具解析，增加可选格式和字节上限。
- backend/src/frontend_ops.rs：Batch桥接、单任务进度、成本门槛、结果采用与取消恢复。
- backend/src/frontend_output.rs：无尺寸猜测的编码与交付文件校验。
- backend/src/frontend_activity.rs、main.rs：新任务和规划期参与切换数据目录的活动检查。
- backend/src/retention.rs：活跃任务引用及完成后24h保留。
- backend/src/transport.rs：进程共享异步运行时。
- backend/src/hybrid.rs：补充4:5/5:4比例校验，保留旧引擎处理路径。
- backend/src/new_product/google.rs、new_product/mod.rs：校验后的原始参考与稳定分析缓存。
- src/components/Assistant.svelte：自然语言一步执行与模式/模型选择。
- src/services/assistant.js、assistant-api.ts、api.ts：统一服务适配。
- src/stores/chat.svelte.js、tasks.svelte.js：上下文快照与单调任务状态。
- src/lib/task-view.ts、src/lib/task-ownership.js、src/services/task-monitor.js：过时响应拒绝、内部任务归属和完成通知。
- src/studio.svelte.js、src/components/CompleteStudio.svelte：托管任务去重，移除旧浮动完成提示。
- src/components/ExecRow.svelte、src/services/commands.js：纯导出不可图片撤销，等待态故障说明。
- src/services/task-output.ts、src/components/AssetLibrary.svelte：按目录及SHA识别已写回结果，重开文件夹不再重复导入。
- docs/frontend-decisions.md：D051–D054；frontend-backend-contracts.md：补充本次契约；frontend-4.11-audit.md：更新实际剩余项。

## 验收
- ✓ 单按钮执行，实时默认，异步Batch和模型选择保留。
- ✓ 同请求费用确认，SelectionContext字段边界验证。
- ✓ 真实后端缓存Batch最终80×100，晚到HTTP不覆盖已完成SSE；无重复公开任务。
- ✓ 新品网格、滚动、澄清、完整应用入口和ZIP夹具回归。
- ✓ coverflow冻结检查通过，仅有已授权只读列表参数。
- ✓ 正式应用更新前无活动任务，旧程序备份，证书保持，数据逐ID/SHA/版本数核对。
- ✗ 外部Windows目录、跨设备队列和全部页面状态/主题等整体P5验收尚未完成，继续按盘点处理。

## 测试与边界
后端121/121；前端纯测试73/73，浏览器条件测试20/20；Svelte检查和发布构建通过。浏览器夹具/缓存回归共22组，无页面错误，不向图像供应商提交。

本轮已有3次真实DeepSeek规划验证：本地旋转+96×128、首页“把 新品/ 的图抠图做 4:5 白底”自动160×200并校验写回、仅PNG保留320×400。真实OpenCV结果白色边角验证通过，原始图片保留。目录为Chromium OPFS授权句柄，不代表外部Windows目录/同事设备已验收。真实图像生成调用0次；Batch测试使用已缓存像素，不能作为供应商Batch生图质量证据。

证据在output/playwright/pipeline/：fixture-receipt.json、batch-receipt.json、deepseek-live-receipt.json、home-natural-receipt.json、deepseek-format-live-receipt.json、deployment-receipt.json。

正式入口http://127.0.0.1:8787/，继续使用一键启动.cmd。OpenCV4.8.0。部署前4项目、63素材；旧文件备份在.tools/deployment-backups/20261028-063929-p4-pipeline/，保留旧哈希静态资源以兼容已打开页面。
