# 4.7 素材库验收

2026-10-27。使用现有Svelte与自写虚拟窗口，无新依赖；已接入P4壳和当前完整应用。

## 改动文件
- src/components/AssetLibrary.svelte：文件夹树、标签、智能分组、筛选、排序、列表切换、右键与批量菜单。
- src/components/AssetGrid.svelte：窗口化网格、rAF滚动合并、单击/Ctrl/Shift/空白框选、三档Ctrl滚轮缩放。
- src/components/AssetTile.svelte：完整比例缩略图、棋盘格、版本/处理/描述/抠图角标；逐个素材响应。
- src/lib/virtual-grid.ts：窗口范围及框选相交纯函数。
- src/services/library.js：缩略图通过hash及thumb Worker处理，复用同一执行入口；mock不生成伪图。
- src/stores/assets.svelte.js：目录前缀、标签、状态及名称筛选。
- src/services/commands.js、workspace.svelte.js、stores/chat.svelte.js：菜单与斜杠共用直接执行函数，结果统一进入执行行，撤销选最新批次。
- src/components/CompleteStudio.svelte、FoundationPage.svelte、src/studio.svelte.js：接入虚拟库，首页/库不整库预取；批量保存建立ID索引避免反复扫描。
- backend/src/frontend_ops.rs：逐素材发出处理状态更新。
- scripts/test-virtual-grid.mjs、verify-library.mjs：窗口、框选、5000素材交互与性能；scripts/studio-test-harness.mjs补齐事件总线的隔离加载。

## 验收结果
- ✓ 5000条素材元数据及不同320×240缩略图：实测60.00 FPS，平均帧16.6653ms、P95 16.70ms。1920×1080时挂载约84张卡片；1280宽度约33张。不是把5000张全部挂载后隐藏。
- ✓ 单击、Ctrl增减、Shift锚点范围、空白拖框、Ctrl+A、输入框Delete保护均通过浏览器验证。
- ✓ 单个asset.updated仅改变对应卡片，约27.4ms；测试先等待滚动窗口与缩略图稳定，避免将最后一帧滚动混入事件观测。
- ✓ 智能分组只读已有标签/描述状态，不调用识别接口；没有描述的素材在未分析组。
- ✓ 右键/批量操作共用执行入口；菜单比例使用RatioBar；URL保存筛选。
- ✓ 1280与1920布局检查、Svelte编译通过；旧P1/P2前端回归16/16通过。

证据：output/playwright/library/report.json、virtual-grid.png、library-1280.png、live-ratio-editor.png。性能是本机Chrome headless夹具数据的实测，不等于所有硬件/原图格式的普遍保证。

## 后续边界
真实识别标签、P5整条自然语言流程、统一旧队列调度、全局四态/无障碍复核尚待后续验收；未标记这些项目完成。一次隔离后端任务曾停在queued，取消及重启后6次完整真实链路通过，未取得可复现根因，保留为P5队列/恢复追踪项，不宣称已经修复。

后续更新：相同任务停滞症状已在场景确认回归中稳定复现，定位到Windows原子替换共享冲突；另修复了快照读取临时记录的问题。定点修复与回归见frontend-4.6.md，不能将其当作全部P5完成。

目录补验：verify-folder-import.mjs使用浏览器OPFS验证原生句柄持久化、子目录保留、重复打开复用同一项目且不重复上传。普通素材导入与新品上限解耦；文件夹打开后建立.gvpak结构。授权恢复与显式选择并发的回归先失败后修复，显式选择优先。
