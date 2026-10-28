# 4.11 导出SelectionContext契约迁移

2026-10-28，已更新正式后台和前端。4.11与P5总体仍未勾选。

## 行为
导出页默认使用真实SelectionContext优先级；选择当前/已选/全部会更新真实选择。预览发送完整8字段上下文及其中的previewIndex，提交发送export操作；不再调用export_run/export_preview/pick_output，不发送旧outputDir、单独ids/assetId或主机路径。编辑参数和导出参数作为数据保持在上下文外，不扩充SelectionContext。

保留PNG/JPEG/WebP、宽高、质量、补边/裁切、背景、大小上限、导出前后对比。旧目录偏好不作为授权使用；用户选择项目保存目录或在结果行下载、另存为。纯导出不新增图片版本、无图片撤销、quotaUnits为0；有效输出写入后读回校验，已有原文件重名时另存编号。部分成功/取消保留交付入口，文件明细分页展示。

## 文件
- backend/src/frontend_export.rs：严格导出参数与只读SelectionContext预览，拒绝旧定位参数、路径、跨工作区和越界索引，复用原Store编码器。
- backend/src/frontend_ops.rs：P4 export增加显式参数，纯导出走同一文件写入器。
- backend/src/frontend_output.rs：保留实际尺寸默认值，使用所选尺寸/适配/质量编码。
- backend/src/frontend_api.rs、main.rs、team.rs：新增只读分派/团队允许项；旧P0–P3编码器及旧命令行为保留。
- src/lib/export.ts：导出参数白名单、范围内预览索引、只含文件名的上下文快照。
- src/services/export.js：导出范围、预览、统一任务及目录选择服务。
- src/components/Export.svelte、ScopePicker.svelte：移除独立范围和旧后端目录路径，保留原参数控件及错误重试。
- src/stores/project.svelte.js：显式更换目录选项，维持原保存授权行为。
- src/services/task-output.ts、src/components/TaskOutputActions.svelte：保留文件夹/下载，补充校验另存为和分页明细。
- src/components/ExecRow.svelte：部分/取消状态同样显示有效文件交付操作。
- src/services/adapters/real.ts、src/stores/chat.svelte.js：采用停止响应快照，修复快任务被误标取消、初始响应晚到的终态转换。
- src/services/adapters/mock-workspace.ts：导出不可用提示清楚，mock继续拒绝真实编码和保存。
- scripts/test-export-contract.mjs：3项参数/完整上下文/旧路径剥离测试。
- scripts/test-task-output.mjs：新增2项另存为时序、取消/校验失败及mock阻止测试。
- scripts/test-history-priority.mjs：新增快速完成停止回归，覆盖running/resolving。
- scripts/verify-export-contract.mjs：7组前端夹具验证，实际OPFS重名保护、ZIP另存为及部分交付。
- scripts/verify-export-contract-live.mjs：隔离真实后台+真实OPFS句柄，全链路及预览字节比对。
- scripts/verify-project-receipts.mjs、verify-region-states.mjs、verify-complete-themes.mjs、verify-complete-mock.mjs：更新对应契约/隔离回归。
- docs/frontend-backend-contracts.md：先记录接口定义；frontend-decisions.md：D066–D068；frontend-4.11-audit.md：同步状态。

## 验收
✓ 预览与提交只有SelectionContext作为素材上下文，字段8项、fileNames无路径；真实全部选择与提交一致。
✓ JPEG80×100质量37，真实预览与下载文件逐字节一致；PNG规范化质量100。
✓ 原图字节、编辑和版本数保持；纯导出undoable:false、quotaUnits:0。
✓ 两个实际返回文件写回并校验；前端夹具另验证原文件重名保护、ZIP另存为及部分交付。
✓ 新只读预览拒绝跨范围/跨工作区及旧参数；旧编码接口保留，无模型调用。
✓ 快任务完成后停止保持done，resolving迟到结果不产生非法状态转换。
✓ real/mock构建与既有关键回归通过；mock导出不申请输出权限、不写目录、不伪造文件。
✓ coverflow冻结、组件服务边界、无阻断反馈检查通过。

## 证据与限制
后台126/126；前端纯测试100/100、浏览器条件38/38，共138项；Svelte及real/mock发布构建通过。真实联调没有付费模型请求。
output/export-backend-tests.log、export-pure-tests.log、export-regressions.log；output/playwright/export-contract/fixture-receipt.json、live-receipt.json、fixture.png、deployment.json。修复前竞态证据export-cancel-red.log、export-resolving-red.log。

文件系统为Chromium OPFS实际句柄，能证明浏览器写入/读回，不代替P5外部Windows目录或同事设备验证；Rust真实后台测试也核对原图磁盘字节。

正式入口http://127.0.0.1:8787/，继续一键启动.cmd，OpenCV4.8.0可用。更新前检查正式/团队任务及规划标记均空闲；备份.tools/deployment-backups/20261028-122059-p4-export-context/。遇到Windows短时exe占用后核实无残留正式进程、旧exe与备份一致，重试替换并恢复服务。4项目/63素材ID、原图SHA、版本数核对不变。

后续仍需全量K状态/无障碍、附属视图主题及P5实际目录/跨账号队列等。此子项不作为全部交付证明。
