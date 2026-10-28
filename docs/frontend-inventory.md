# 4.0 现有前端盘点与实施计划

日期：2026-10-27。4.0盘点记录；用户已回复「继续」、批准页面路由回退，并选择保留原模块监听、外部快捷键统一管理。4.1–4.8已完成，详见相应frontend-4.x.md。4.9进行中。下文4.0历史基线和只读边界描述均指盘点当时，数据列表注入阻塞仍未解决。

## 1. 仓库与本次边界

- 实际 Git 根目录：本文件所在 `docs/` 的上一级，即 `GVPAK Studio` 仓库。上一级是工作目录，不是 Git 仓库。
- 盘点基线：`200bc46`（Unify GVPAK Studio naming across runtime, tooling and documentation）。
- 开始时受控文件无修改；已有两个未跟踪审阅资料目录：`给Claude审阅的代码/`、`Claude核心代码TXT/`。它们不是运行源码，本次未修改。
- 受控文件共 164 个：`src/` 40 个文件，其中 Svelte 组件 22 个；`backend/src/` 13 个文件；`scripts/` 55 个文件；`docs/` 12 个文件。其余是配置、启动入口、图片、构建产物和可执行程序。
- 阅读和核对范围：入口、全部前端组件与共享状态、图像/绘画/下载辅助模块、样式和构建配置；后端命令分发、存储类型、处理流程、AI 请求、账号权限、SSE、队列、保留机制；已有契约、设计记录及测试入口。二进制、字体和图片只核对用途，不作为可编辑源码；不把审阅副本或历史生成脚本当成当前实现。
- 用户附件先按原始字节保存到 `docs/frontend-spec.md`，之后只将 4.0 清单改成已完成。P0–P3、4.1–4.11、P5 均不勾选。
- 本轮只新增/修改文档，不修改业务源码、依赖、构建产物、后端、coverflow 模块或任何运行数据。

## 2. 技术栈结论：沿用现有实现

| 项目 | 当前事实 | 后续决定 |
| --- | --- | --- |
| 框架 | Svelte 5；锁文件实际版本 5.57.2 | 沿用 Svelte 5 和 runes，不迁移 |
| 构建 | Vite 7；锁文件 7.3.7；Svelte 插件 6.2.4 | 沿用现有 Vite 配置和 npm 锁文件 |
| 语言 | 主要为 JavaScript；已有 `src/cost.ts` | 旧文件不批量改写；规格要求的 `target.ts`、`ratio.ts`、`shortcuts.ts` 等新增纯逻辑使用 TypeScript |
| 状态 | `studio.svelte.js` 的 `$state` 单例；组件 `$derived` / `$effect`；`layout.svelte.js` 单独管理尺寸 | 将新职责逐步放入 8 个 Svelte 状态模块；保留兼容入口，避免重复维护同一份状态 |
| 路由 | 无路由库；`studio.mode` / `studio.view` 条件渲染，没有 URL 路由同步 | 采用 History API 的小型路由模块，接入既有 App 入口；深链接服务支持另列后端缺口 |
| UI / 图标 | 原生 HTML 控件、自写 Svelte 组件、`lucide-svelte` 0.468.0 | 复用已有图标与控件模式，补充基础组件，不引入 UI 组件库 |
| 样式 | `src/style.css` 全局样式为主；少量组件局部样式；CSS 变量与普通选择器 | 增加 `src/styles/tokens.css`、基础样式，保留现有目录和 coverflow 样式边界 |
| 字体 | Arial/Helvetica 系统栈，品牌使用 Barlow Condensed 5.3.0 | 新正文采用规格系统字体；沿用已经安装的品牌字体资源 |
| 实时通信 | `EventSource('/api/events')`，后端 SSE | 继续使用 SSE，不另接一套 WebSocket |
| 测试 | `node:test` + `assert`；Svelte compiler；已有隔离状态测试 harness | 沿用，不新增测试依赖；当前 Node v24.14.1 可执行新增的可擦除类型 TS 纯函数测试 |

不采用从零项目的推荐组合：本仓库已有可工作的 Svelte 应用和图像编辑能力，切换 React/Zustand 会违背用户约束，并导致 coverflow 模块、编辑状态和测试体系再次迁移。新增目录仅承接新职责，不重建另一套应用。

### 构建与运行

- `src/main.js` → 登录态检查 `/api/team/session` → 动态导入并挂载 `App.svelte`；入口 HTML 为 `index.html`。
- `npm run dev`：Vite 监听 127.0.0.1:5173，`strictPort: true`；`/api` 代理至 127.0.0.1:8787。
- `npm run build`：输出 `dist/`；当前 `dist/` 受 Git 管理。
- `npm run check`：调用 Svelte compiler 检查 App、组件和主状态模块；它不是完整 TypeScript 类型检查。
- `npm run test:p1` / `npm run test:p2`：现有前端回归命令；这些名称是历史分组，不能当成新规格 P1/P2 已通过的证据。
- 发布运行：`start.cmd` → `start.ps1` → `bin/gvpak-studio.exe`，Rust/Axum 提供 `dist/` 与 HTTP/SSE；团队 HTTPS 为另一个服务入口。
- 原生服务当前用 `ServeDir` 回退，未看到任意应用 URL 回退 `index.html` 的明确配置；开发服务器可打开深链接不代表发布服务能刷新 `/editor/:assetId`。

## 3. 目录与可复用代码

| 路径 | 当前职责与接入判断 |
| --- | --- |
| `src/App.svelte` | 顶栏、页面条件渲染、工具栏、全局按键、拖入、提示/对话框和 coverflow 挂载；适合在此演进 AppShell，不能另起应用 |
| `src/studio.svelte.js` | 项目/素材/选择/草稿/历史/任务/通知混合单例；导入、保存、还原、SSE 都在这里；是拆分职责与兼容现有组件的主要边界 |
| `src/layout.svelte.js` | 右栏与 coverflow 高度、折叠、视口、localStorage；coverflow 使用的字段及尺寸函数必须保持 |
| `src/api.js` | 统一 invoke 请求、Blob URL、JSON/base64 上传、错误翻译、本地设置；已有组件也有直接 fetch，后续需逐步归口 services |
| `src/components/` | 21 个组件，见下表；没有第二个前端包 |
| `src/preview-cache.js` / `preview-loader.js` / `gpu-preview.js` | 内存 LRU、预览去重/取消/优先级、WebGL2 交互预览；可复用，当前不是按 hash 持久化的缩略图 IndexedDB 缓存 |
| `src/paint.js` / `radial-layout.js` | 笔迹坐标与栅格化、菜单几何；画笔能力可保留，旧径向浮层不直接进入新主流程 |
| `src/coverflow-selection.js` | coverflow 模块的 Space 选择手势纯逻辑；属于冻结边界 |
| `src/local-downloads.js` | IndexedDB 目录句柄/收据、读写校验、自动下载、确认保存；可复用底层机制；旧 confirm 和清理语义不能原样带入新交互 |
| `src/export-preview.js` / `grid-click.js` | 导出预览生命周期、双击识别；复用合适的纯逻辑，不复用新规格禁止的交互步骤 |
| `src/assistant-intent.js` / `assistant-presets.js` | 旧指令补充和快捷指令；其当前语义不等于新规格白底/抠图指令，不直接照搬 |
| `src/cost.ts` / `branding-migration.js` | 费用显示与旧设置迁移；保留旧数据兼容，新参数集中管理 |
| `src/style.css` | 紧凑的全局样式和多次覆盖；含 coverflow、画布、对话框、径向菜单和响应式规则；不能整文件替换 |
| `backend/src/main.rs` / `transport.rs` / `team.rs` | 命令入口、本机 HTTP/SSE、团队账号/HTTPS/队列/下载/权限；本轮只读 |
| `backend/src/core.rs` / `paint.rs` | 项目素材、原图/版本/回收、图像处理和笔迹；版本类型以实际 Rust 源码为准 |
| `backend/src/assistant.rs` / `hybrid.rs` / `opencv_worker.py` | 样图后批处理、DeepSeek JSON 计划、混合后台任务与 OpenCV；不能当成新工具调用契约直接使用 |
| `backend/src/models.rs` / `image_provider.rs` / `branding.rs` / `retention.rs` | 模型目录、图片生成、凭据、团队临时保留与清理；本轮不改 |
| `docs/command-contract.json` / `docs/source-contracts/index.ts` | 旧契约快照；缺少后续字段/命令，不能替代当前 dispatch 和 Rust 类型 |
| `scripts/` | 现有测试、检查、打包与历史源码替换脚本；只运行验证命令，不运行旧替换脚本 |
| `public/` / `backend/demo/` | 示例图片；新组件不直接硬编码示例业务数据 |
| `dist/` / `bin/` / `output/` / `node_modules/` | 构建、可执行程序、验证输出、依赖；不是本轮开发对象 |

### 组件清单

| 现有组件 | 当前用途 / 后续处理 |
| --- | --- |
| `Coverflow.svelte` | coverflow 模块，原样保留；完整接口见第 4 节 |
| `Canvas.svelte` | 缩放、平移、裁切、原图对比、CPU/GPU 预览；逐步适配编辑器 |
| `PaintLayer.svelte` | 独立笔迹与精确取色；保留可用底层能力，不把普通笔迹宣称为已有抠图掩膜 |
| `Inspector.svelte` | 调整、裁切、图层/历史、保存和还原；演进为工具/属性/版本入口 |
| `Library.svelte` | 项目列表、搜索、勾选、回收和导入；网格未虚拟化，缺少 Shift 锚点/拖框选择 |
| `Assistant.svelte` | 旧处理方案、样图、后台任务列表；复用能力接线，交互改为 ChatPanel 与执行行 |
| `Generate.svelte` | 文字/参考图生成，结果会打开旧 Review；新流程需直接产生版本与执行行 |
| `Export.svelte` | 导出参数与报告；底层导出能力可复用，新结果进入执行行 |
| `Settings.svelte` | 密钥、尺寸、费用、快捷键与数据目录；已有显式保存按钮，尚非五组自动保存设置 |
| `TeamBar.svelte` / `TeamAdmin.svelte` / `Login.svelte` | 团队下载、账号管理和登录；复用权限依据，需接入新的 session 状态 |
| `ResizeHandle.svelte` | 指针捕获、键盘调尺寸、双击复位；coverflow 所使用的原行为冻结 |
| `ScopePicker.svelte` | 旧当前/选中/项目范围；由新 SelectionContext 和来源文字替代 |
| `Confirm.svelte` / `Review.svelte` | 旧模态确认和结果预览；不作为新 ConfirmRow 复用 |
| `ProjectPeek.svelte` | 项目悬停预览浮层；不加入新主流程 |
| `RadialMenu.svelte` / `ArcSlider.svelte` | 旧径向工具与弧形参数；当前有独立键盘监听，不直接作为规格允许的浮层复用 |
| `Histogram.svelte` | 延迟计算 RGB 直方图，可按需要复用 |
| `Feedback.svelte` | 旧反馈浮层和直接 fetch；不借本轮扩展功能范围 |

### 状态与样式现状

`studio` 混合保存项目、素材、`pid` / `aid`、选择、草稿、单图 undo/redo、旧模式、busy/progress、旧弹窗和 jobs。`assets()` 仅按当前项目和回收状态过滤；素材库搜索在组件内部计算。`scope()` 没有 @ 引用、编辑器/筛选四级优先级，也没有标准 SelectionContext。

新 8 个 store 的职责对应：session（登录/主机/配额）、project（项目/目录/授权）、assets（索引/筛选/单项更新）、selection（有序选择/锚点/SelectionContext）、tasks（进度/排队/结果）、chat（对话/消息/执行行）、editor（素材/版本/工具/比例/属性）、prefs（尺寸/视图）。采用 `src/stores/*.svelte.js`，通过 Svelte 响应式保持单一事实来源；不把整个 `studio` 快照来回复制。coverflow 的旧共享状态兼容方式须先解决第 5 节阻塞。

现有颜色变量是 `--red`、`--line`、`--muted`、`--paper`、`--stage`，缺少规格完整 token 与系统深色主题。有基础 `:focus-visible` 和局部 reduced-motion；全局按钮动画 160ms、右栏 180ms、coverflow 350ms/600ms 等不能宣称已符合新验收。后续新 UI 使用不超过 150ms；coverflow 的原版时间不改，其与通用动画要求的冲突单独记录。

## 4. coverflow 模块的实际接口

### 4.1 运行版本与历史参考必须区分

当前运行的是 `src/components/Coverflow.svelte`，由 `App.svelte` 以 `<Coverflow/>` 挂载。挂载条件为编辑模式、画布视图、非专注模式。**它没有 `$props()`、公开组件方法、`bind:` 数据接口、可订阅的自定义切图事件或对外回调属性。**

`docs/CoverFlow-reference.tsx` 是历史 React 参考，未被入口导入，且没有对应 React 运行依赖。不能因为该参考有参数，就声称当前模块已有这些参数；也不可以重新启用它来替代现有模块。

### 4.2 当前数据输入方式：共享状态与导出函数

```text
studio.projects + studio.pid + studio.trash
  → current() → assets() → 模块内部 list
studio.aid → list 中的当前索引（找不到时索引为 0）
studio.thumbs[asset.id] → <img src>
studio.selected → 勾选与计数
current().name → 标题
layout.coverflowHeight / coverflowCollapsed → 高度与收起状态
```

| 输入/依赖 | 数据形状或条件 | 实际含义 |
| --- | --- | --- |
| `assets()` | `current()?.assets.filter(a => a.trashed === studio.trash) || []` | 当前项目中指定回收状态的完整数组；保留项目顺序；没有选中/搜索/本批结果参数 |
| 素材记录 | 模块直接读 `id: string`、`name: string`；过滤还依赖 `trashed: boolean` | 薄宿主不能删除共享资产其他字段；旧 `select()` 还要读取 `edit` 和草稿 |
| `studio.pid` / `projects` | 项目 ID、项目 `{id,name,assets,...}` 数组 | `current()` 从中确定项目 |
| `studio.aid` | 当前素材 ID | 与 URL 无自动联动 |
| `studio.thumbs` | `Record<assetId, imageURL>` | 原图/Blob URL 由外部准备，模块不调用下载接口 |
| `studio.selected` | `string[]` | 勾选可包含列表以外 ID；标题总数使用全局 selected 长度，全选判断只看当前列表 |
| `studio.busy` / `modal` / `confirm` | 真值会阻止部分操作 | 与旧操作锁/弹窗耦合；不同入口的判断并不完全相同 |
| `studio.mode` / `view` | 快选要求严格等于 `'edit'` / `'canvas'` | 新路由不能直接把这两个值改成 URL 页面名 |
| `layout.coverflowCollapsed` | boolean | 收起后释放快选手势 |
| `layout.coverflowHeight` | 默认 144，持久化 | 高度样式使用 `Math.min(height, coverflowMax())` |
| `coverflowMax()` | `max(100, min(360, viewportHeight - (viewportWidth<=650?76:48) - 22 - 220))` | 原版自适应高度上限；分隔条 min 为 100 |

可以通过现有共享状态设置项目、当前图、缩略图和选择；**不能在不改变共享项目内容/数据源函数的情况下，为当前模块单独传入另一个显示列表。** 仅设置 `studio.selected` 会改变勾选，不能改变显示列表。

### 4.3 输出、事件与内部回调

| 操作/事件 | 当前调用或写入 | 外部可观察结果 |
| --- | --- | --- |
| 点图片、箭头、滚轮、拖动、快选导航 | `select(id)` | 写 `studio.aid`；装载对应 draft；重置 pan/zoom/compare/preview；保存草稿位置；没有 `onChange` 事件 |
| 点图片勾选框 | `toggle(id)` | 切换 `studio.selected`，`batchScope='auto'`；不切换当前图 |
| 点全选 | 模块内部 `selectAll()` | 当前列表全选/取消当前列表选择，保留列表外选择；busy 时禁用 |
| 点击项目名 | `studio.view='projects'` | 进入旧项目列表；没有 `onOpenProjectLibrary` 对外回调 |
| 点折叠按钮 | 写 `layout.coverflowCollapsed`、`saveLayout()` | 持久化展开/收起 |
| 拖动分隔条 | ResizeHandle 的绑定 `value`、内部 `onstart` / `onfinish` / `reset` | 展开、高度改变、结束时保存；双击恢复 144px 并展开 |
| `window keydown/keyup` | 模块 `onMount` 注册，capture=true | 快选处理部分键；销毁时移除 |
| `window blur` / `document visibilitychange` | `release()` | 页面失焦/隐藏释放快选；普通键盘焦点改变并非同一个事件 |
| pointerleave / pointercancel | hovered=false + `release()` | 鼠标离开/指针取消结束快选 |
| 项目/busy/modal/confirm/折叠状态的 effect | `quick.release()` | 同步结束选择手势；项目变化也会触发重新执行 |

`createQuickSelection(options)` 是模块内部辅助接口，不是 coverflow 的组件参数。其现有参数/回调为：

| 名称 | 签名 | 当前用途 |
| --- | --- | --- |
| `getIds` | `() => string[]` | 获取模块内部 list 的 ID 顺序 |
| `getCurrent` | `() => string` | 返回 `studio.aid` |
| `isSelected` | `(id) => boolean` | 判断共享选择 |
| `setSelected` | `(ids: string[], checked: boolean) => void` | 批量加入/移除共享选择，去重并设 batchScope |
| `navigate` | `(id: string) => void` | 绑定 `select` |
| `onHeld` | `(held: boolean, selecting: boolean) => void`，默认空函数 | 绑定内部 `modeNotice`，显示勾选/取消提示 |

返回 `held`、`selecting` 只读 getter，以及 `press()`、`release()`、`step(-1|1)`、`wheel(delta)`。一次 Space 按住期间锁定“选中”或“取消选中”意图，经过已处理图片不会反转；两端夹紧，不循环。

### 4.4 按键表：以当前源码为准

模块所谓“可用”是**鼠标悬停**，不是焦点判断：`hovered && !typing() && !busy && !modal && !confirm && !collapsed && mode==='edit' && view==='canvas'`。`typing()` 检查当前 activeElement 的 contenteditable、input、textarea、select、`[role=textbox]`。根 section 本身没有 tabindex，不能把规格中的“聚焦”直接等同于当前实现。

| 按键/场景 | 当前行为 | 处理层级与冲突 |
| --- | --- | --- |
| Space，满足上述可用条件 | 首次 keydown 翻转当前图勾选并开始快选；repeat 不重复翻转 | 模块 window capture，preventDefault + stopPropagation；优先于旧 App 画布 Space |
| Space + ← / → | 每个 keydown（含自动重复）前/后一张，并沿用同一勾选意图 | 模块 capture 消费；失去可用条件则 release 并放行 |
| 松开 Space，快选进行中 | release，清理累计滚轮量 | 模块 keyup capture 消费；选择结果保留 |
| 单独 ← / →，未持有快选 | 模块自己的 capture handler 不处理；旧 App 编辑画布作用域调用 `step()` 切图 | 此导航在 App，不可误记为 coverflow 对外回调；单独方向键在旧 App 无边界循环 |
| Esc | 模块 capture 总是先 release，再放行 | 未 prevent/stop；旧 App 随后处理弹窗/工具状态；本版不等于“停止任务” |
| Ctrl / Alt / Meta + 键、IME composing | 模块 keydown 不接管（Esc 分支在这些 guard 前） | Shift 本身不被该 guard 排除，因此 Shift+Space/方向仍可能进入快选 |
| 1–5、R、Delete | 模块没有对应绑定 | 新页面绑定仍须服从输入焦点与作用域；不凭空给模块新增按键 |
| 高度分隔条聚焦：↑ / ↓ | 高度 +10 / -10 px；Shift 为 +40 / -40；min/max 夹紧 | `ResizeHandle.svelte` 元素级 keydown，消费事件 |
| 高度分隔条聚焦：Home / End | 到最小/最大高度并保存 | 消费事件，操作时展开模块 |
| 按钮聚焦：Enter / Space | 保留原生按钮激活；Space 也可能被悬停快选 capture 拦截 | 后续不得通过新监听暗中重写当前差异 |

宿主旧按键还包括 Ctrl+S、Ctrl+Z/Ctrl+Shift+Z、Ctrl+1–4、B、Q/E、0、Backslash 按住对比；App 当前只检查 input/textarea/select，并非完整的 contenteditable 检查。这些属于后续快捷键归口的迁移范围，不能当成新规格已实现。

### 4.5 鼠标、布局与样式基线

- 普通滚轮取 `deltaY || deltaX`，累计严格大于 35 或小于 -35 才切相邻一张并清零；单个大事件不会跨多张。
- Space+滚轮先按 deltaMode 转换（行×16，页×240），辅助逻辑在反向时清掉旧累计量，每个过阈值事件只走一张；与方向键混用会清理残余量。
- 左键拖动起点忽略勾选框/侧箭头和 busy；位移小于 7px 不开始；拖动每累计 65px 切一张（一次移动可多步），向左拖下一张，向右拖上一张；拖动结束抑制那次点击。
- 索引两端夹紧，按钮边界禁用；空列表计数为 000/000。当前 ID 不在列表时视觉索引退到 0，但模块不会自动修复 ID。
- 透视 1000px；中心 Z=60、scale=1.06；两侧位移 `sign(offset)*(130+(abs(offset)-1)*75)`、yaw 为左 +44° / 右 -44°、Z=-40×abs、scale=max(0.68,0.95-abs×0.06)；仅渲染偏移绝对值≤7 的卡片。
- 高度默认 144px，收起 34px，沿用原版 resize 规则；卡片大小由 `--cf-height` 的 clamp 计算。样式在全局 `src/style.css`，不在组件局部 style 中。
- 卡片 `object-fit:cover`，350ms transform；侧卡白色 wash 覆盖层；快选提示释放后 600ms 淡出。原版快选提示不等于规格要求的任务状态角标。
- CSS 先将 cf-stage/cf-stack 设为 pointer-events:none，文件末尾再将 cf-stage 恢复为 auto；不能只复制前面一段 CSS 判断交互。
- 新处理状态层只能放在原模块 DOM 外侧、`pointer-events:none`；按素材 ID 对齐，并跟随原卡片几何/可见性。没有现成 slot 或卡片节点 ref，定位实现要在接入方案确定后验证，不能往原模板插节点。

### 4.6 历史参考的参数、事件与回调（不属于当前运行 API）

| 历史 TSX 参数 | 类型/默认值 |
| --- | --- |
| `assets` | `{id:string,name:string,previewUrl:string}[]` |
| `activeAssetId` | `string | null` |
| `selectedAssetIds` | `string[]` |
| `projectName` | `string` |
| `onActiveAssetChange` | `(id:string) => void` |
| `onToggleSelection` | `(id:string) => void` |
| `onOpenProjectLibrary` | `() => void` |
| `keyboardEnabled` | `boolean`，默认 true |

参考版的左右键由 window effect 注册，忽略 input/textarea；没有当前 Svelte 版的 Space 快选、拖动和调整高度接口。其回调不能直接传给 `<Coverflow/>`；给未声明的参数赋值不会自动接通数据。

### 4.7 后续回归对照表

- [x] 源码核对数据输入、输出、副作用、按键、鼠标、几何和布局。
- [x] 运行现有选择/键盘/拖动测试：17/17 通过。
- [ ] 4.9 接入后逐项实机对照：点击、勾选、全选、边界、普通滚轮、Space 点按、Space+滚轮/方向自动重复、选择意图、释放、拖动反向、分隔条键盘/复位、折叠/项目跳转。
- [ ] 同时验证列表来源、URL 同步和新增外层状态角标；这些属于后续新要求，本轮未宣称通过。

### 4.8 冻结指纹（SHA-256）

| 文件 | SHA-256 |
| --- | --- |
| `src/components/Coverflow.svelte` | `df628550eb7b4b1184828325ce5ec713c689412760e6e756b0bdcf130ab5ed3e` |
| `src/coverflow-selection.js` | `d9d53debb2dd0c09f9a6842056e9fb4f5e445f1a6b8ae6a54cf3b316672d9039` |
| `src/components/ResizeHandle.svelte` | `97d11753d15101e65a36b7c094c29290e890042f29d7d07bc7618a5a25577b3d` |
| `src/layout.svelte.js` | `554faf0bfedb4f8c0bab1f4be9bd3b472f4532e4b08464a1bca309269d656b25` |
| `src/style.css`（本轮全文件基线） | `10af828ef5af7c4492feef6d1790c2bb606bc36bcb36289c555c80e863a17036` |
| `docs/CoverFlow-reference.tsx` | `be1addb67a179fa0530f182c3361db693330a9a275306a4070a446e2ce0597b7` |
| `docs/coverflow-reference.css` | `edaf2d84e3a3ed2bceb4c3f30b22e14157f9d69c99c63ab11d52a6fbdcd00911` |

后续可修改全局样式的其他部分，但 coverflow 相关规则、继承颜色、通用控件规则对模块的影响也应回归验证；不能用全局覆盖绕过“不改样式”。兼容的状态拆分也不得改变上述尺寸函数与 resize 行为。

## 5. 必须汇报的 coverflow 接入阻塞

1. **没有独立列表输入接口。** 新版要求显示选中项/筛选/本批结果，而实际 `assets()` 只能返回当前项目按 trashed 过滤的完整素材。更改 `assets()` 返回语义会影响旧 Library/Canvas；裁剪或临时替换 `studio.projects` 会污染共享索引，不是安全的数据传入方式；本轮均未执行。
2. **快捷键集中管理与冻结冲突。** 模块内部自己注册 capture keydown/keyup，分隔条也自己处理按键；无 keyboardEnabled 参数。新 `shortcuts.ts` 可以管理外部动作并让位，不能在不改模块的情况下成为这些原有监听的唯一注册点。悬停与规格“聚焦”也不一致。
3. **通用视觉要求与冻结冲突。** 全局动画≤150ms、主题 token 若覆盖原模块，将改变其现有 350ms/600ms 时长或视觉样式。不能默认为允许统一覆盖。

结论：可继续独立基础设施和 mock 数据层的准备，但不能承诺仅写一个传数据的 `CoverflowHost.svelte` 就能满足全部接入验收。**等待用户决定这些边界；在决定前不实现该宿主，不改模块、其辅助选择逻辑或按键/样式。** 若用户提供同技术栈且已有上述接口的原版源码，应先重新核对其接口再接线；不自动切回历史 React 参考。

## 6. P0–P3 当前后端与联调缺口

以下是源码盘点，不是对新规格后端验收通过的声明。本轮未启动新服务、调用 AI、导入图片或改动后端。

| 分组 | 已有事实 | 对新规格的缺口 / 联调策略 |
| --- | --- | --- |
| P0 | Windows keyring 状态/写入/删除；本机 8787 与团队 HTTPS 8788 隔离；团队命令白名单；证书生成和设备登录 | 未找到规格 quota/threshold/settings/certificate 指纹与安装脚本接口；keyring 现状不能证明“旧明文→DPAPI→删除”的验收；需契约确认 |
| P1 | `bootstrap`、项目 CRUD、素材 sha256、版本、回收/还原、24h 团队保留；浏览器下载目录句柄已存 IndexedDB | 当前上传 JSON/base64，先上传再在服务端去重；不是二进制预查且零重复传输；没有所需文件夹层级/标签/描述元数据和批量事务 undo/redo；现有主机存储不是浏览器项目 `.gvpak/` 结构 |
| P2 | `assistant_plan` / `hybrid_plan`；本地 resize/flatten/auto_center 等；样图与 apply；已有生成与导出 | 未找到 `describe_assets`、语义 cutout、标准 `fit_ratio`、预设管线、斜杠执行接口；flatten 仅填透明背景，不等于抠图；计划当前传统计信息、返回 JSON 方案，不是 SelectionContext + function calling |
| P3 | SSE 事件包 `{event,payload}`；`job-progress`、`hybrid-*`、`team-queue`；hybrid_cancel(jobId)、全局 cancel_job | 进度常为 `{kind,done,total}`，无统一 taskId；team-queue 只含 state，无数值位置/主机优先保证；缺少标准 task/asset 事件；不能在前端伪造真实队列位置 |
| 部署 | `ServeDir(dist)`、Vite dev 回退 | 发布版四条 URL 的直接访问/刷新需要明确 SPA 回退支持 |

### 已有调用契约摘录

- `POST /api/invoke/{name}`，JSON，`X-GVPAK-IPC: 1`；图片命令可返回 octet-stream；错误体通常 `{error:string}`，团队权限拒绝可返回 403。
- `bootstrap {}` → `{projects,warnings,dataRoot,profile,version}`。
- `thumbnail {projectId,assetId}` / `preview_source {projectId,assetId}` → Blob。
- `edit_commit {projectId,assetId,edit}` / `restore_version {projectId,assetId,versionId}` / `asset_trash {projectId,ids,restore}` → Project。
- `assistant_plan` / `hybrid_plan {projectId,ids,instruction}` → 旧方案；后端自行构造 dimensionCounts 等上下文，不能仅在请求上补一个 context 就假定 DeepSeek 已收到它。
- `assistant_preview {projectId,ids,plan}` → `{sampleId,before,after,...}`；`assistant_apply {sampleId,outputDir}` 依赖已有样图记录，不能用其冒充直接执行接口。
- `hybrid_start {projectId,ids,instruction,recipe,outputDir,approved,clientRequestId}` → Job；`hybrid_jobs {}` → `{jobs,notices}`；`hybrid_cancel {jobId}`。
- `export_run {projectId,ids,preset}` → 导出报告；`cancel_job {}` → 无 taskId 的旧取消。
- `GET /api/events` → SSE，数据为 `{event,payload}`；事件语义与新规格不同。

### 待确认的后端接口定义草案

以下只用于审阅，**未建立接口、未修改 dispatch、未把这些名字当成已经可调用的真实接口**。可由后端采用等价命名；需确认字段与语义后才能做依赖它们的真实联调。项目 ID 是业务定位参数，素材选择上下文只用规格 F 的 SelectionContext，不另发路径或第二种选择快照。

```ts
type Ratio = '1:1' | '3:2' | '4:3' | '4:5' | '16:9';
type Orientation = 'landscape' | 'portrait';
type Operation =
  | { kind: 'describe_assets' }
  | { kind: 'cutout' }
  | { kind: 'white_bg' }
  | { kind: 'center' }
  | { kind: 'fit_ratio'; ratio: Ratio; orientation: Orientation;
      long_edge?: number; mode?: 'auto' | 'pad' | 'crop' }
  | { kind: 'export'; format: 'png' | 'jpeg' | 'webp' }
  | { kind: 'preset_white_bg'; ratio: Ratio;
      orientation: Orientation; long_edge?: number };

// SelectionContext 字段完全使用 frontend-spec.md 的 F 定义。
// filter.folder 使用项目内部标识；fileNames 是名称，不是路径。
type ExecuteRequest = {
  projectId: string;
  context: SelectionContext;
  operations: Operation[];
  clientRequestId: string;
  confirmToken?: string;
};
type TaskState = 'queued' | 'running' | 'done' | 'partial' | 'failed' | 'cancelled';
type Task = {
  id: string; accountId: string; type: string; targetCount: number;
  state: TaskState; done: number; total: number;
  queuePosition: number | null; startedAt: number | null;
};
type Gate = { reason: 'describe_count' | 'generation_cost_or_count' | 'quota';
  message: string; confirmToken: string };
```

| 草案命令/传输 | 请求 → 返回与必要语义 |
| --- | --- |
| `session_status` | `{}` → `{account:{id,name},isHost,isLocal,quota:{used,limit}}`；由服务端提供权限与真实配额 |
| `assets_list` | `{projectId,filter:SelectionContext['filter']}` → `{assets,folders,total}`；素材含 hash/名称/尺寸/标签/描述/抠图/版本/处理状态；文件夹用内部 ID；只返回必要元数据 |
| `operation_execute` | `ExecuteRequest` → `{task:Task}` 或 `{confirmation:Gate}`；复用标准操作，直接命令不经过 DeepSeek；一次执行生成可撤销的版本/批次记录 |
| `chat_execute` | `{projectId,instruction,context:SelectionContext,clientRequestId,confirmToken?}` → 同上，并有方案/流式文字事件；DeepSeek 仅用 SelectionContext 和工具定义，不沿用尺寸统计上下文 |
| `tasks_list` / `task_stop` / `task_cancel` | `{}` → `{tasks:Task[]}`；`{taskId}` → `{task:Task}`；运行停止与排队取消区分，目标隔离，Esc 一秒内生效 |
| `batch_undo` / `batch_redo` / `version_activate` | `{projectId,batchId}` → `{assets,currentVersionIds}`；激活 `{projectId,assetId,versionId}` → 更新素材；保留后继版本，支持 50 张一批恢复原字节 |
| `upload_probe` + 二进制上传 | `{projectId,files:[{name,sha256,bytes}]}` → 缺少的 hash/已存在 ID；仅缺失项发送字节，导入目标为逻辑 `导入/YYYY-MM-DD/`；具体传输格式由后端确认 |
| `settings_get` / `settings_patch` | 只在主机本机开放；读取不含密钥；patch 只提交变更字段（模型/配额/门槛/费用上限或密钥写入）；返回脱敏设置，远端 403 |
| `certificate_info` / `certificate_regenerate` / 安装脚本下载 | 主机本机限定；返回指纹和受控下载地址，绝不返回私钥 |
| 现有 SSE 扩展 | `task.progress`、`task.queue`、`task.done`、`task.failed`、`asset.updated`，每项含 taskId/assetId 及相关数值；正常取消也需任务状态更新；聊天增量事件另确认 |
| 发布服务路由 | `/`、`/library`、`/editor/:assetId`、`/settings` 返回应用入口；`/api` 和不存在的静态资源保留原错误语义；不能以 URL hash 替代规格路由 |

适配原则：在 services 层将已证实的旧能力映射为新前端数据形状；未实现能力用显式 mock，不调用不存在的命令。建议环境开关 `VITE_FRONTEND_DATA_MODE=mock|real`，未知值报错，真实模式缺能力显示可重试错误，不静默回退假结果。设置/凭据和付费请求不纳入自动验证。

## 7. 规格歧义与决定

全部记录在 `docs/frontend-decisions.md`。关键点：

- 优先使用用户本次明确约束；原文保留，不擅改整份规格。
- “筛选超过 50 张”的纯函数测试仍覆盖全部 ID 与数量；遵守最新“只有三种门槛”的要求，不因数量单独新增第四种 ConfirmRow，不截断/丢弃目标。计数信息作为普通执行行说明。
- 当前前台批次完成后选中该批结果并同步当前图，使完成后的结果展示落在“选中优先”规则内；后台其他批次通过“查看结果”进入，不抢走当前操作。不新增替代组件。该决定尚受真实列表输入阻塞限制。
- 当前 coverflow 的默认高度、手势、动画和内部监听按实际源码盘点，不暗中按通用规范重写；冲突由用户明确处理。

## 8. 分阶段实施计划（尚未执行）

| 顺序 | 计划改动位置与交付 | 验收/测试重点 |
| --- | --- | --- |
| 4.1 | 现有 `App.svelte` 演进 AppShell；新增 `src/styles/`、`src/ui/` 和轻量路由；保留旧组件目录 | 系统主题即时切换；1280px 无横向溢出；focus-visible；刷新恢复项目/筛选/栏宽；真实深链接受后端回退支持约束 |
| 4.2 | `src/services/` HTTP/SSE/fs/hash 与 mock；`src/stores/*.svelte.js` 八个职责；`src/workers/`；`src/lib/target.ts`、`ratio.ts` | target 四种来源、空集/去重/顺序、超过50不丢ID、fileNames无路径；ratio 五比例×两方向、默认/交换/1:1、长边留空不放大；纯函数单元测试必写 |
| 4.3 | `src/lib/shortcuts.ts` 作用域栈与冲突表；移出现有宿主/组件零散按键到统一入口 | 输入框/IME/可编辑元素、页面/全局优先级、释放、1–5/R/Delete保护；冲突表必写单测；模块冻结冲突未解决时不改其监听 |
| 4.4 | 原组件目录中顶栏、授权 Banner、TaskIndicator/TaskDrawer | SSE 合并、授权恢复不跳页、配额数值与任务分组；缺接口的 UI 测试使用 services mock |
| 4.5 | ChatPanel、五种卡片、Composer、MentionPicker、SlashMenu；`lib/slash.ts`、执行行状态机 | 状态机必须单测；普通/斜杠路径、三种确认、↑先撤销再执行、@、拖入、Esc、历史滚动；组件不造数据 |
| 4.6 | HomePage、快捷指令、最近项目/任务 | 发送即进入素材库、保留同对话、关联选择；导入日期目录和@引用 |
| 4.7 | 演进 Library；文件夹树/筛选/虚拟网格/角标/批量与右键菜单；复用下载底层能力 | 自写窗口化网格，无新增依赖；5000素材、Shift/框选/Ctrl+A、单项更新、hash缓存、智能分组不请求AI |
| 4.8 | RatioBar 共用一份；编辑器与菜单使用同一调用 | 默认方向、⇄实际文字、1:1禁用、新版本/批撤销；无额外比例入口 |
| 4.9 | 演进 Canvas/Inspector，添加版本条与属性；仅在接口问题解决后添加薄 `CoverflowHost.svelte` | 比较/缩放/画笔/版本；逐项对照第4节；外层 pointer-events:none 状态；切图与URL同步；不改模块内部 |
| 4.10 | 现有 Settings 演进五组设置 | 主机本机权限、远端跳回/403、自动保存与字段旁反馈、密钥只写不回显 |
| 4.11 | 补全状态与无障碍；性能计时/浏览器验证 | 每区域四态、键盘主流程、1280/1920与浅深主题截图；量测60fps/首屏1秒/选择100ms/斜杠300ms/事件200ms；达不到就报告数值与原因 |

执行顺序严格为 4.1 → 4.2 → 4.3 → 4.4 → 4.5/4.6/4.7/4.8 → 4.9 → 4.10 → 4.11；4.5–4.8 可逐项完成，不要求并行。每项完成再自查、勾选、按用户格式回报。最后按 P5 用例在真实后端逐项验收；mock、构建成功或旧测试不能代替真实联调。

## 9. 4.0 验收与验证记录

- ✓ 已保存附件全文，只勾选任务 4.0。
- ✓ 已列出实际框架/版本、构建/运行、目录、状态管理、样式、组件和后端接线现状。
- ✓ 已写明 coverflow 模块真正的数据入口、无公开参数的限制、内部事件/回调、按键和鼠标表；历史参考单独标识。
- ✓ 决定沿用 Svelte 5 + Vite + Svelte 状态机制，未新增依赖。
- ✓ 现有模块测试：`node --test scripts/test-coverflow-selection.mjs scripts/test-coverflow-keyboard.mjs scripts/test-coverflow-drag.mjs`，17 通过、0 失败。
- ✓ `npm run check`：App、21 个子组件和主状态模块编译通过。
- ✓ 业务源码/后端/配置/脚本/构建产物相对基线无 diff；冻结文件指纹一致。
- 本轮不新增实现单测：4.0 只改文档，复用现有测试确认行为基线。target、ratio、快捷键冲突表、执行行状态机的单测分别在 4.2/4.3/4.5 交付。
- 未执行会写入受控 dist 的构建、付费调用、真实运行 UI 验收、P0–P3 新接口联调或性能验收；这些均不在 4.0 的通过声明中。
- 需要确认：第 5 节 coverflow 边界与第 6 节后端契约草案。新增依赖需求：无。

下一步：停止在 4.0，等待用户回复「继续」；继续仅开展已明确的后续范围，涉及模块内部、后端契约或新增依赖的工作仍按用户规定先确认。


## 4.9 授权变更附注
用户明确允许coverflow仅增加可选只读items列表参数，默认undefined继续使用assets()。数据之外的事件、回调、按键和样式不变。去掉新增props声明并恢复list表达式后，源码SHA-256仍为df628550eb7b4b1184828325ce5ec713c689412760e6e756b0bdcf130ab5ed3e。4.0表格是历史基线；新的薄宿主与状态覆盖层见frontend-4.9-progress.md。
