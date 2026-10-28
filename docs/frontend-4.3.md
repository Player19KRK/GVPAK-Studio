# 4.3 快捷键系统验收记录

日期：2026-10-27。状态：4.3已完成。用户已选择保留原版监听，shortcuts.ts统一管理模块外部按键并让原版优先。

## 任务编号与名称

4.3 快捷键系统，对应规格H。

## 改动的文件

| 文件 | 一句话说明 |
| --- | --- |
| `src/lib/shortcuts.ts` | 快捷键定义、作用域栈、冲突表、输入/IME保护、长按释放和DOM统一注册入口 |
| `src/App.svelte` | 注册应用动作并删除旧keydown分支；切图与URL同步期间保持原模块实例挂载 |
| `src/components/AppShell.svelte` | 对话输入框支持聚焦和草稿，以便Ctrl+K和Ctrl+J实际工作 |
| `src/components/Canvas.svelte` | 画布固有Enter/Esc通过统一注册，并在输入控件中让位 |
| `src/components/Inspector.svelte` | 数值滑块的键盘快照通过统一注册 |
| `src/components/Library.svelte` | 原素材按钮Enter操作通过统一注册 |
| `src/components/ArcSlider.svelte` | SVG滑块原有方向键/Home/End行为通过统一注册 |
| `src/components/RadialMenu.svelte` | 原有菜单Tab/Esc和按钮激活通过统一注册，不新增浮层 |
| `src/components/Feedback.svelte` | 原反馈区域Esc通过统一注册，不改变反馈流程 |
| `src/components/ProjectPeek.svelte` | 原预览Esc通过作用域栈注册，不把该旧预览加入新主流程 |
| `src/services/commands.js` | 比例、删除与撤销动作使用共同服务入口；没有真实接口的能力保持显式错误 |
| `src/services/workspace.svelte.js` | 记录接受的任务并通过当前适配器按ID停止，注入mock时不会切回真实适配器 |
| `src/stores/chat.svelte.js` | 增加输入草稿，收起/展开对话栏时保留文字 |
| `src/stores/tasks.svelte.js` | 跟踪有效的活动任务，任务列表刷新后清理过期活动ID |
| `scripts/test-shortcuts.mjs` | 11项作用域、冲突、输入保护、IME、长按和异常单元测试 |
| `scripts/test-shortcut-history.mjs` | 验证已有编辑草稿撤销/重做与新入口兼容 |
| `scripts/verify-shortcuts.mjs` | 浏览器验证输入、原模块、分隔条、作用域栈、导航与任务停止 |
| `docs/frontend-spec.md` | 仅勾选任务4.3 |
| `docs/frontend-decisions.md` | 记录用户选择、键盘归属和后续业务接线边界 |
| `docs/frontend-inventory.md` | 更新当前阶段，不覆盖4.0的历史证据 |
| `docs/frontend-4.2.md` | 将原先待确认的监听边界更新为用户已确认 |
| `docs/frontend-4.3.md` | 本任务逐文件说明、验收和测试记录 |

## 按键与作用域

| 作用域 | 按键 | 动作 |
| --- | --- | --- |
| 全局 | Ctrl+K / Ctrl+J | 聚焦并必要时展开对话输入框 / 收起展开对话栏 |
| 全局 | Ctrl+Z / Ctrl+Shift+Z | 统一历史动作；有批次记录时走批次服务，否则保留旧草稿撤销/重做 |
| 全局 | Esc | 先交给顶部临时作用域，未消费时停止当前任务；取消取色 |
| 素材库 | Ctrl+A / Delete / Enter | 全选当前筛选 / 删除动作服务 / 打开当前选择 |
| 编辑器 | 1–5 / R | 按规格顺序选择比例 / 交换方向；1:1交换无动作 |
| 编辑器 | Space | 长按对比原图，释放恢复原来的对比状态 |
| 既有编辑器兼容 | Backslash、Ctrl+S、←→、B、Q/E、0 | 原图对比、保存、切图、画笔、工具切换、适配居中 |

- 默认优先级：输入保护 → 原模块/原分隔条及控件固有行为 → 临时作用域栈 → 页面 → 全局。
- 输入框中1–5、R、Delete、Space、左右键等不触发应用动作；Ctrl+Z/Ctrl+A等保留原生文本行为，应用只保留Esc、Ctrl+K、Ctrl+J。
- 输入编辑器可显式注册↑↓/Enter/Tab导航用于后续@/斜杠菜单；这不放开Delete或比例快捷键。
- IME组合输入、AltGraph、已消费事件不会再次触发命令。
- 普通按键不因自动重复反复提交；左右导航允许自动重复。
- 按住对比支持Space与Backslash重叠，失焦、隐藏、切换页面/素材或进入输入区域时释放，避免卡住。
- 九个原模块相关按键的归属和六个不冲突的编辑键，完整写入`coverflowConflictTable`；不额外制造与原版冲突的映射。

## 验收结果

- ✓ 输入焦点下按1–5/R/Delete不起应用作用，覆盖textarea、input、select、contenteditable、role=textbox及Shadow DOM输入。
- ✓ 原版coverflow的Space选择、Space+左右键自动重复、反向、释放、Esc、普通方向切图均在真实DOM验证通过，没有双步。
- ✓ 原版高度分隔条的方向/Home键正常；外部宽度分隔条的键盘操作只执行一次。
- ✓ Ctrl+K聚焦/展开对话，Ctrl+J每次只切换一次，草稿保留。
- ✓ 多层作用域按后进先出处理Esc，关闭后恢复全局动作。
- ✓ Space对比在松键、失焦、焦点进入输入框后正确恢复，组合长按也不遗留状态。
- ✓ Ctrl+A只选择当前筛选，Enter同步编辑器URL；文字内撤销不误触发图片撤销。
- ✓ Esc在输入框聚焦时仍能通过现有适配器按任务ID停止任务；测试请求仅在本机夹具中拦截，没有操作真实任务。
- ✓ 源码扫描发现的直接keydown/keyup监听仅剩lib/shortcuts.ts及用户明确保留的Coverflow.svelte/ResizeHandle.svelte。
- ✓ 原模块七个冻结文件、样式、配置、依赖和发行文件没有改变；本任务未修改后端。

## 测试

- `test-shortcuts.mjs`：11/11，包括冲突表、输入保护、IME、原模块优先、作用域、自动重复、长按与异常。
- `test-shortcut-history.mjs`：1/1，保持已有编辑草稿撤销/重做兼容。
- 原有前端58项继续通过，合计70项通过。
- `verify-shortcuts.mjs`：10组浏览器检查通过，pageErrors=[]。
- 4.1基础设施7组浏览器回归、4.2数据层4组浏览器回归继续通过。
- Svelte编译检查无错误/警告；生产构建输出至隔离目录，不覆盖当前受控dist。
- 报告：`output/playwright/frontend-shortcuts/report.json`。

浏览器回归发现宿主用当前素材ID与URL短暂不一致作为卸载条件，打断原模块Space手势；已在App外层改为验证目标存在性，保持模块实例连续。原模块内部文件未改动。

## 自行做的决定

- 严格按用户答复保留原模块及分隔条监听，不拦截后重放事件，也不改原版悬停/焦点差异。
- 菜单和控件固有键盘行为通过统一action注册，组件可以提供动作回调，不能各自安装DOM键盘监听。
- 非冲突的1–5/R在原模块聚焦时仍可交给编辑器；Space及原生按钮/分隔条按键优先保留给原所有者。
- 没有批次记录时保留已有草稿撤销。比例/删除/真实批次效果依赖后续业务任务和已确认的后端契约，本任务只验证动作分发及缺能力时的明确失败，不宣称生成了新版本。
- 首页输入框属于4.6；当前Ctrl+K在已具备输入骨架的素材库/编辑器中完成接线。对话输入框目前保留草稿，发送/@/斜杠仍在4.5完成。

## 需要确认的事项

本任务无。原模块的数据列表注入问题仍保留为4.9前的独立阻塞，本次按键选择不构成修改其数据接口或样式的授权。

## 下一步

4.4顶栏、授权提示条、任务抽屉。后续仍遵守4.5–4.8→4.9→4.10→4.11的顺序。
