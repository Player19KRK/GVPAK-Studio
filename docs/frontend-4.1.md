# 4.1 基础设施验收记录

日期：2026-10-27。当前状态：4.1已完成并勾选。用户已明确批准页面路由回退，已实施并通过真实HTTP和后端回归测试。

## 已实现

- 继续使用现有 Svelte 5、Vite、App 入口与组件目录，无新增依赖。
- 设计变量包含浅色/深色颜色、间距、字号、圆角、运动时长和棋盘格；直接使用系统媒体查询，主题变更无需刷新。
- 9 个基础组件：Button、IconButton、Segmented、Badge、Tooltip、Resizer、Banner、Spinner、Kbd。交互控件支持 hover、active、disabled、2px focus-visible；静态 Badge/Kbd 等不增加无意义 Tab 停靠点。
- Tooltip 采用浏览器原生 title，图标按钮有 aria-label，快捷键同时出现在提示中，不新增浮层卡片。
- AppShell 为 48px 顶栏 + Grid 主区/对话栏；对话栏默认400、范围360–520、收起为0并保留24px展开把手；文件夹树默认220、范围180–320。
- History API 实现首页、素材库、素材编辑、设置及未知地址；导航和前进后退同步，筛选保留五个查询参数。
- localStorage 按账号隔离，恢复项目、筛选、栏宽与收起状态；读取异常/损坏数据时使用默认值，保存失败不阻断操作。
- 设置入口仅在服务返回本机主机身份且浏览器地址为回环主机名时显示，团队会话访问设置回到首页。此处是前端入口限制，不替代服务端403检查。
- 编辑器继续使用原有 Canvas/Inspector 与原版 coverflow 模块；当前 ID 与 URL 同步。此阶段未实现选中/筛选/本批结果列表注入，未完成4.9。
- 没有挂载旧版模态确认、结果预览、项目悬停预览或成功 toast；首页、素材库、对话及设置目前是4.1布局骨架，业务交互分别在后续任务完成。
- 旧属性面板的历史入口在新壳中暂时禁用，避免触发本轮未挂载的旧确认/Review；4.9接入新的可撤销版本条后再启用。旧Canvas的图片列表和模块项目入口通过已有状态输出导航到素材库。
- 新 DOM 键盘监听只在 `lib/shortcuts.ts` 注册；Resizer 和旧宿主通过 action 接入。4.3作用域/冲突表尚未实现；原模块和其他旧组件的内部监听没有改动。

## 改动的文件

| 文件 | 改动 |
| --- | --- |
| `src/main.js` | 加载新主题/基础样式；读取本机访问依据；连接失败仍显示可重试应用壳 |
| `src/App.svelte` | 在现有入口接入AppShell、路由/项目恢复，保持既有编辑模块，移除旧页面外壳与弹窗挂载 |
| `src/app/shell.svelte.js` | Svelte响应式壳状态、导航、popstate和本地保存 |
| `src/lib/router.ts` | 路由解析、查询参数和URL编码、设置入口资格判断 |
| `src/lib/preferences.ts` | 本地偏好白名单、尺寸夹紧、存储读写与损坏恢复 |
| `src/lib/shortcuts.ts` | 提供统一DOM键盘注册入口，暂不声称完成4.3 |
| `src/styles/tokens.css` | 浅色/深色、尺寸、字号、圆角、运动和透明棋盘变量 |
| `src/styles/base.css` | 新控件/布局/焦点样式，排除旧编辑器通用覆盖，保持原模块样式 |
| `src/ui/Button.svelte` | 基础按钮与disabled/busy/样式变体 |
| `src/ui/IconButton.svelte` | 带可访问名称和快捷键提示的图标按钮 |
| `src/ui/Segmented.svelte` | 分段按钮组、当前状态及禁用支持 |
| `src/ui/Badge.svelte` | 状态文字角标及语义色 |
| `src/ui/Tooltip.svelte` | 原生提示与快捷键文案 |
| `src/ui/Resizer.svelte` | 指针拖动、方向键/Home/End、双击复位及尺寸ARIA |
| `src/ui/Banner.svelte` | 内联提示、错误和操作区域 |
| `src/ui/Spinner.svelte` | 加载状态、辅助文字与减少动画支持 |
| `src/ui/Kbd.svelte` | 快捷键文字样式 |
| `src/components/AppShell.svelte` | 顶部导航、真实项目切换、对话栏骨架和宽度/收起控制 |
| `src/components/FoundationPage.svelte` | 首页/素材库/设置布局与加载、错误、未知页面骨架 |
| `src/components/Inspector.svelte` | 新增可选historyEnabled入口，旧使用者默认不变，新壳暂不开放依赖旧模态的历史操作 |
| `backend/src/transport.rs` | 增加明确的页面GET路由及HTTP测试，本机服务接入相同静态页面服务 |
| `backend/src/team.rs` | 团队服务接入已批准的同一静态页面服务；账号/API处理逻辑不变 |
| `docs/frontend-spec.md` | 勾选4.1，未勾选后续任务 |
| `scripts/check-svelte.mjs` | 递归检查新增UI组件和Svelte状态模块 |
| `scripts/test-frontend-foundation.mjs` | 路由/设置入口/偏好持久化单元测试 |
| `scripts/fixtures/FoundationUI.svelte` | 9种UI组件的独立测试夹具，不进入应用入口 |
| `scripts/verify-frontend-foundation.mjs` | 浏览器尺寸/主题/导航/存储/控件验收，服务响应只在测试路由拦截器中构造 |
| `docs/frontend-decisions.md` | 增加4.1实现决定和待确认的发布路由问题 |
| `docs/frontend-inventory.md` | 更新阶段指针，4.0历史盘点结论保留 |
| `docs/frontend-4.1.md` | 本任务交付、测试和限制记录 |

## 验收结果

- ✓ 系统主题即时跟随：同一个已打开页面切换light/dark，计算背景色立即变化，无需重载。
- ✓ 1280px无横向滚动条：四个路由×浅/深主题；补充1920px，共16个视图，document.scrollWidth均等于视口宽度。
- ✓ 新交互控件focus-visible：独立组件页逐个聚焦，均为2px solid；Resizer支持方向键、Home/End及双击恢复。
- ✓ 本地恢复：项目、筛选、对话栏宽度与折叠、树宽在刷新后保留，前进/后退正确。
- ✓ 新控件减少动画：系统reduce时Spinner动画为none。
- ✓ 模块与后端边界：7个冻结文件、package/lock、dist/bin均未修改；后端仅改用户已批准的页面路由。
- ✓ 原生页面路由：真实Axum/ServeFile HTTP测试验证带查询参数/编码ID的页面返回HTML；缺失API/资源404、原API403、页面POST405、HEAD无正文均保留。

## 测试

- `node --test scripts/test-frontend-foundation.mjs`：4/4。
- coverflow选择/键盘/拖动回归：17/17。
- `npm run test:p1`：8/8；`npm run test:p2`：8/8。它们是历史回归分组，非新规格后端验收。
- `npm run check`：全部Svelte组件与状态模块编译通过。
- `npm run build -- --outDir output/playwright/frontend-build`：构建通过；不覆盖当前受控dist。
- Playwright浏览器验收通过，pageErrors=[]；17张截图和机器报告在 `output/playwright/frontend-foundation/`。运行时使用本机已有Playwright与Chromium，没有安装包或浏览器。
- 第一轮独立组件夹具有Svelte运行时导入不一致和重复选择器，已修正并复验。测试使用生成的有效PNG，不向任何真实后端/AI发送请求。
- 设计机械检查发现旧编辑器沿用Arial；这是保留现有视觉基线的预期，不为通过风格建议更换字体。
- Rust后端回归55/55通过，含新增页面路由HTTP测试。初始沙箱临时文件写入/清理被拒绝；改用仓库内隔离临时目录并通过授权运行后全数通过，未改存储/清理业务逻辑。

## 自行做的决定

详见 `frontend-decisions.md` 的D011–D014：原生Tooltip、原生项目选择控件、先建立统一键盘注册入口、新样式排除旧模块、构建到隔离目录。

## 需要确认

本任务无剩余确认。用户批准的路由已实现，采用明确页面路由，不使用通配HTML回退。现有bin和dist没有部署替换，正式发行打包仍在后续阶段。

## 下一步

进入4.2数据层；未完成的后端能力采用显式mock适配，真实接口调整仍须确认。coverflow列表输入和内部按键问题仍未得到修改授权。
