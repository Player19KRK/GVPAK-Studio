# 4.11 旧反馈入口迁移

2026-10-28，已更新正式完整应用，4.11/P5总体仍保持未勾选。

## 改动文件
- src/studio.svelte.js：notify改为本机结果事件；按项目保存草稿，restoreVersion兼容入口转标准版本操作，删除旧确认处理。
- src/stores/chat.svelte.js：本机已完成结果进入执行行，项目恢复动作保存ID/工作区，撤销失败可重试；本机结果不走云重试。
- src/components/ExecRow.svelte：本机结果、撤销动作及导出明细，保留原任务执行状态机。
- src/components/ExportResults.svelte：从原导出页移出的文件结果和保存/复制操作，仍展示实际输出或失败原因。
- src/components/Export.svelte：将真实报告送入对话，保留输出参数、原目录及原编码接口，不再重复展示独立结果区。
- src/components/Library.svelte：删除项目直接回收，先保存该项目草稿；持久项目恢复回执。
- src/components/CompleteStudio.svelte：移除toast及旧Confirm/Review渲染。
- src/components/Confirm.svelte、Review.svelte：删除已失去活动调用入口的旧组件，备份在.tools/source-backups/receipts/。
- src/components/Inspector.svelte、PaintLayer.svelte：去除旧恢复/取色提示入口，透明位置给出错误，颜色控件直接反映选择。
- src/components/ChatPanel.svelte：常规云重试排除本机结果。
- src/local-downloads.js、src/components/TeamBar.svelte：浏览器下载后的清理改为选择已保存文件，核对大小及SHA再确认；取消、不匹配或缺少读取能力不清理。
- scripts/test-history-priority.mjs：新增3项本机回执单测，含只执行一次、失败重试、工作区隔离。
- scripts/test-local-downloads.mjs：更新原生确认测试，保留原8项目录、写入/关闭、校验、重开、账号、另存为测试。
- scripts/verify-project-receipts.mjs：真实浏览器夹具，原版会触发confirm断言；新流程验证草稿保存、回收、刷新撤销、导出与失败。
- scripts/verify-project-recycle-live.mjs：隔离真实后台回收/恢复，核对编辑、版本历史、原图字节及SHA。
- scripts/check-feedback-contract.mjs：源码回归检查原生confirm/alert、toast渲染及旧组件引用。
- docs/frontend-decisions.md：D062/D063；frontend-4.11-audit.md：同步未完成接口适配范围。

## 验收
✓ 删除项目没有原生确认；草稿先保存到版本，项目回收后原图保留。
✓ 项目撤销跨刷新可用；真实后台恢复后编辑/版本及原图字节完全一致。
✓ 导出成功/失败及文件动作统一在对话执行行，失败不会触发模型重试。
✓ 完整源树没有window.confirm/window.alert、成功toast渲染、旧Confirm/Review组件。
✓ 同事端普通浏览器下载不能仅凭点击就清理：必须选择实际已保存文件，匹配字节及SHA后才能发saved回执。授权目录和另存为保留自动读回校验。取消/错文件不确认清理。
✓ 原生保存/打开文件选择器属于文件授权操作，不是确认弹窗；只在点击时请求。
✓ coverflow冻结检查通过，无后端逻辑/接口或依赖变更。

## 测试
纯测试87/87、浏览器条件37/37，共124项；Svelte检查、生产构建通过。项目/导出5组专项，普通生图6组、新品7组以及既有配置、区域、执行行和编辑器回归均通过，无页面异常、付费请求0。

真实后台实验只创建并恢复.tools/cloud-smoke-dev/data中的专用测试项目，正式项目未变更。证据：output/playwright/receipts/fixture-receipt.json、recycle-live.json、results.png、deployment.json；output/receipt-pure-tests.log、receipt-tests.log、receipt-download-red.log。失败的原生确认与恢复校验路径有可重跑脚本。

普通旧回调撤销仍属于本页面会话能力；项目恢复使用明确元数据，刷新后可恢复。本机结果是已经发生操作的记录，不伪造后台任务、费用或图片版本。其数据不进入DeepSeek；处理请求仍沿用SelectionContext。

## 部署及剩余范围
http://127.0.0.1:8787/，一键启动OpenCV4.8.0可用。正式4项目/63素材ID、原图SHA、版本数核对一致，服务入口与构建一致。只更新静态前端，旧index备份.tools/deployment-backups/20261028-091828-p4-receipts/。
工具会话更新期间检测到正式/测试端口均无进程监听，已按原配置恢复正式启动；未停止进行中进程或改数据目录。

导出页底层仍沿用旧export_run/export_preview，尚需服务层适配和完整mock验收。其他K状态/无障碍、附属视图主题、真实设备与跨账号队列等P5保持开放，不用本子项证据标记全部交付。
