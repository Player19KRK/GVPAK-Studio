# 4.5 ChatPanel 验收记录

2026-10-27，沿用Svelte、目录和已批准的后端契约，无新依赖。

## 文件与职责
- src/components/ChatPanel.svelte：对话切换、消息流、历史滚动保护、新消息按钮。
- src/components/ChatText.svelte、ExecRow.svelte、AssetGridCard.svelte、ConfirmRow.svelte、ErrorCard.svelte：五种卡片；文本安全转义，执行进度、撤销、导出、结果选择、失败重试与一行确认。
- src/components/Composer.svelte：最多8行、Enter发送、Shift+Enter换行、@搜索/键盘选择/整块删除、斜杠菜单、↑编辑、导入和Esc停止；按键从shortcuts.ts统一分发。
- src/stores/chat.svelte.js：持久会话与执行行、流式事件、重发前撤销、确认token、失败项重试、完成素材更新。
- src/services/chat-files.js：复用导入服务并转@引用；日期文件夹归类在4.6验收。
- src/lib/execution.ts、slash.ts：纯状态机、三种门槛、6个命令、自然语言撤销；精确匹配的完整命令一次Enter即发送。
- src/components/AppShell.svelte、CompleteStudio.svelte：接入同一个ChatPanel；完整应用Ctrl+K/J、可调宽度；属性工具与对话切换。
- backend/src/frontend_ops.rs：持久任务、本地操作、按ID取消、批次撤销重做、幂等、重启恢复；新操作队列优先选择等待中的主机任务。
- backend/src/frontend_chat.rs：DeepSeek流式工具调用，仅传instruction与SelectionContext，服务端验证工具参数后执行。
- backend/src/frontend_api.rs、main.rs、team.rs、transport.rs：增量分发、团队工作区边界及任务恢复。
- backend/src/core.rs：增加批次状态恢复辅助方法；历史保留，原图不覆盖。
- backend/src/opencv_worker.py：增加本地前景分割输出alpha；白底先分割再合成，不把不透明照片原样填白冒充抠图。复杂多主体/不确定轮廓会失败并报告。
- src/services/new-product.ts：导入后补齐原版coverflow需要的缩略图数据，不改模块内部。

## 验收
- ✓ 纯状态机覆盖完整状态序列、四种终态、终态不可回退、直接命令跳过planning、三种门槛。
- ✓ 斜杠/自然语言撤销只请求operation_execute，不请求DeepSeek；竖16:9输出9:16。
- ✓ ↑修改上一条先撤销旧批次，再提交新命令；撤销失败时不重新执行。
- ✓ @搜索、chip整体删除、导入后@引用、流式文字、Esc精确停止、确认token/取消分支均有浏览器证据。
- ✓ 历史滚动位置保留，新消息显示跳转按钮。
- ✓ 真实隔离后端完整应用：新品导入→编辑器对话→比例输出→新版本→撤销；检查尺寸、原图哈希、原基底与版本数，未调用DeepSeek/生成接口。
- ✓ 原版coverflow缩略图完整加载，7个冻结文件哈希与4.0一致。

## 测试与交付
- 74/74后端全量回归通过（包括真实OpenCV alpha/白底像素、持久任务恢复、幂等、批次撤销重做和本机流式provider夹具）。
- 35项相关前端单测通过；Svelte编译通过，无新增编译警告。
- scripts/verify-chat.mjs 7组浏览器检查通过；scripts/verify-chat-live.mjs真实隔离后端4组检查通过；新品完整应用6组回归通过。
- output/playwright/chat/live-complete-app.png 保存完整应用截图；用户原数据未用于写入测试。
- 已更新原bin/dist并通过原一键启动链启动，OpenCV 4.8.0可用。

## 后续边界
这次不把真实DeepSeek服务质量、复杂商品抠图质量、5000素材性能、50张批量逐字节回滚、旧队列与新增队列的统一主机优先、配额计量当作通过；属于后续P4/P5验收。UI的配额确认用模拟事件验证，当前未配置账号配额。原coverflow列表接口许可仍待用户答复。

DeepSeek协议依据：https://api-docs.deepseek.com/guides/tool_calls/ 、https://api-docs.deepseek.com/api/create-chat-completion/ 。未新增SDK。
