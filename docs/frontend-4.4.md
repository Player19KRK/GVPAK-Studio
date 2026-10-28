# 4.4 顶栏、授权提示条、任务抽屉

2026-10-27，按用户批准的 frontend-backend-contracts.md 添加会话/任务读取与停止接口。前端同时接入 P4 AppShell 和当前完整应用，不替换原版 coverflow。

## 文件
- src/components/TaskCenter.svelte：任务指示、配额、运行/排队/最近完成分组、按任务停止和结果入口，Esc关闭。
- src/components/FolderAuthorization.svelte：仅在已有目录句柄授权失效时显示32px提示，原地授权，focus/visibility复核。
- src/components/AppShell.svelte、CompleteStudio.svelte：共享入口；完整应用使用原 SSE。
- src/services/task-monitor.js、src/lib/task-view.ts：读快照、归一化事件、分组、未知值、配额阈值；没有新增依赖或第二条实时连接。
- src/stores/project.svelte.js、session.svelte.js、src/studio.svelte.js：授权复核、标准会话字段、连接事件。
- backend/src/frontend_api.rs、main.rs、team.rs：增量 session_status/task_list/task_stop；团队工作区隔离，终态停止幂等，返回任务快照不含原始上下文和主机路径。
- src/services/adapters/real.ts、mock.ts：标准接口接线，保留mock环境开关。
- scripts/test-task-view.mjs、verify-task-center.mjs、fixtures/TaskUI.svelte：纯函数与浏览器验收。

## 验收
- ✓ 实时任务/队列事件只更新对应任务；配额更新从68%到97%触发警告，严格大于95%。
- ✓ 目录授权失效显示提示，重新授权后消失，URL不变；测试在浏览器OPFS隔离目录，未操作用户目录。
- ✓ 停止按具体taskId；运行与排队分组独立，Esc关闭并返回入口焦点。
- ✓ 完整应用新品流程、滚动、生成确认、结果ZIP和自动摆正入口回归通过。
- ✓ 10项前端单测、67项后端回归通过；浏览器5项专项检查通过，无pageerror。

## 真实联调边界
现有旧任务没有真实数字排队位置/开始时间，保持null并显示待提供。当前账号没有配额账本/上限配置，session_status返回quota:null，界面显示未提供，不能拿mock数值声明真实配额已实现。用户批准的P3主机优先队列、配额计量以及operation_execute/chat_execute将在后续接口工作完成，P5尚未通过。本任务勾选代表前端4.4实现及适配验收，不代表P0–P3/P5完成。

## 发布验证
原8787服务检查时未运行；备份bin与入口后更新原bin/dist，使用start.ps1 -NoBrowser通过一键启动链启动。session_status实测isHost/isLocal为true，task_list实测读取10个已有任务。新品完整应用6组浏览器检查回归通过。配额仍为null，不宣称真实计量完成。
