# 4.11 进行中：本地步骤、结果写回与整批撤销

本轮子项已完成并部署；4.11和P5总体仍不勾选。

## 改动文件
- src/components/Assistant.svelte：保留11类手动步骤、预设和输出参数，改为直接执行；取消本地样图预览/应用步骤；统一显示新旧任务；本地规划走SelectionContext。
- src/services/assistant.js、assistant-api.ts：本地计划执行、预设、配置和规划服务，含显式mock适配。
- src/services/task-output.ts：按授权目录写回、双重SHA256校验、按账号保存凭据、同名另存、多文件一次ZIP下载。
- src/stores/output.svelte.js：结果保存状态、自动恢复、同任务事件合并，已撤销任务不自动重新写回。
- src/components/TaskOutputActions.svelte：保存/下载/检查与错误反馈，明确已保存文件不会被版本撤销删除。
- src/services/task-monitor.js：任务事件和重开列表接入自动写回。
- src/components/ExecRow.svelte、src/stores/chat.svelte.js：对话执行行接收输出描述、状态和保存动作。
- src/services/adapters/real.ts、mock.ts：真实结果取图，mock明确拒绝返回假图片。
- backend/src/frontend_output.rs：独立交付文件和描述、实际字节数/哈希校验、任务所属工作区取图；不暴露路径。
- backend/src/frontend_ops.rs：local_plan处理、默认交付文件、导出失败警告、持久结果事件、进程worker及批次恢复。
- backend/src/core.rs：新增整批状态恢复方法，一次项目状态提交，暂存/恢复派生缩略图；旧单图/导入/回收逻辑保留。
- backend/src/frontend_api.rs、main.rs、team.rs：assistant_context_plan和operation_result的新增路由/团队白名单。
- scripts/test-task-output.mjs、test-assistant-api.mjs：文件校验、同名保护、重开凭据、并发序列化、权限失效、ZIP唯一文件名与上下文隔离。
- scripts/studio-test-harness.mjs、test-p1-regressions.mjs：原样图协议回归转为直接执行边界，继续验证草稿不被提交动作清空及参数快照。
- scripts/verify-local-plan.mjs：步骤/预设/规划/执行行的完整应用fixture回归。
- scripts/verify-local-plan-writeback-live.mjs：真实后端+Chromium目录句柄写回、关闭页面恢复与撤销。
- scripts/verify-batch-50-undo-live.mjs：50张不同PNG的处理、导出与逐字节撤销。

## 本轮验收
✓ 本地步骤不再要求样图预览，不出现确认弹窗或成功toast，参数在服务端严格验证。
✓ 每张一个新版本，导出尺寸即最终处理尺寸；无法达到字节上限时保留版本并明确失败原因，不返回不合格文件。
✓ 取结果检查任务归属/大小/哈希；交付描述没有主机路径，较新编辑不会被结果重导出或采用覆盖。
✓ 已授权目录自动写回，权限失效不自行弹窗，读取本机保存结果再验哈希；同名原文件保留。
✓ 重开页面恢复句柄和保存凭据，确认过的文件不重复下载；多文件ZIP内重名会另取名称。
✓ 标准任务已纳入Assistant任务列表，旧混合/Batch任务能力仍保留。
✓ 整批恢复先校验全部目标，一次提交状态；中断恢复不覆盖无关新编辑，原始图片和版本文件不改写。
✗ 自然语言混合/Batch的旧规划和启动仍待迁移；全应用四态/主题/mock/进度顺序和P5剩余项继续，未声明完整交付。

## 测试与实际证据
- Rust104项全通过，包括新计划、输出损坏拒绝、字节上限、较新编辑保护、Windows项目文件读占用、部分撤销恢复和既有回归。
- 前端纯逻辑66项、P1/P2/export/新素材事件17项通过。
- 本地步骤fixture浏览器5组检查；新品生产构建7组回归通过。
- 真实写回：通过项目授权界面使用Chromium OPFS FileSystemDirectoryHandle；原始original.webp保持，结果保存为original (2).webp和original (3).webp，哈希与服务端一致。页关闭事件时间1793134022519，服务端任务完成时间1793134022926，确证执行中关闭；重开自动写回，再刷新无重复下载。两次undo恢复原图预览字节和源文件SHA256。此证据不替代外部Windows目录测试。
- 真实50张：一个任务完成50张及50个输出；一次undo后全部当前base/edit恢复，50个原文件与上传fixture逐字节一致；本次处理加撤销约3100ms（仅本机64×80合成PNG，非大图吞吐保证）。此前实测中断的1个合成批次也已恢复。
- 输出证据：output/playwright/local-plan/live-writeback-receipt.json、batch-50-undo-receipt.json；output/local-plan-backend-tests.log、local-plan-js-tests.log、local-plan-frontend-tests.log。
- 本轮付费模型提交0。本地语言规划的浏览器测试使用fixture；没有把其当作实际DeepSeek规划质量验收。

## 部署
主机和团队未完成任务计数均为0后，备份并更新正式bin/dist，经原start.ps1启动。OpenCV4.8.0正常；4个项目、63张素材的ID/原图SHA256/版本数一致，正式证书哈希不变，exe/index与验证构建一致。

自行决定：D048–D050。无新依赖。
需要确认：无。
下一步：迁移自然语言混合/Batch管线，接着全应用4.11状态/性能检查，再完成P5逐条验收。
