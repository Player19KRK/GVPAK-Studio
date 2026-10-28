# 4.2 数据层验收记录

日期：2026-10-27。状态：4.2已完成；当前总进度见frontend-spec.md清单。

## 任务编号与名称

4.2 数据层，对应规格E、F。

## 改动的文件

| 文件 | 一句话说明 |
| --- | --- |
| `src/lib/target.ts` | 纯函数解析四级来源，构造/序列化SelectionContext并清除文件名中的路径 |
| `src/lib/ratio.ts` | 五个基础比例、默认方向、交换、实际标签和像素尺寸换算 |
| `src/lib/preferences.ts` | 扩展网格尺寸档位与视图偏好的白名单持久化 |
| `src/services/api.ts` | real/mock选择、唯一上下文和请求字段白名单 |
| `src/services/http.ts` | HTTP请求、invoke、Blob响应及错误状态处理 |
| `src/services/adapters/real.ts` | 接入现有只读接口、旧任务投影和明确的缺能力错误 |
| `src/services/adapters/mock.ts` | 集中提供隔离的模拟工作区/任务和可控事件，不调用HTTP或AI |
| `src/services/realtime.ts` | 单一SSE连接、事件总线、旧事件映射及关闭/重连反馈 |
| `src/services/normalize.ts` | 保留资产对象身份并补充索引字段，缺少的服务端信息维持unknown/null |
| `src/services/fs.ts` | 目录句柄恢复、显式重新授权、.gvpak结构和避免覆盖的文件写入 |
| `src/services/idb.ts` | 目录句柄和缩略图共用的IndexedDB事务操作 |
| `src/services/hash.ts` | SHA-256 Worker调用与释放 |
| `src/services/thumbnails.ts` | 缩略图Worker调用、尺寸档位和释放 |
| `src/services/worker-client.ts` | Worker请求编号、并发结果匹配、错误和结束清理 |
| `src/services/workspace.svelte.js` | 启动/重试、八个store协调、单项事件更新及销毁 |
| `src/workers/hash.worker.ts` | 在Worker内计算SHA-256，不在组件里处理摘要 |
| `src/workers/thumb.worker.ts` | OffscreenCanvas/createImageBitmap缩略图生成及按hash缓存 |
| `src/stores/session.svelte.js` | 账号、主机/本机依据、真实配额或未知值、加载状态 |
| `src/stores/project.svelte.js` | 当前项目、最近项目、文件夹句柄及授权状态 |
| `src/stores/assets.svelte.js` | 共用资产索引、筛选/排序、有序结果和单项更新 |
| `src/stores/selection.svelte.js` | 共用有序选择、锚点、@引用、范围选择及rAF上下文更新 |
| `src/stores/tasks.svelte.js` | 按任务ID合并进度/排队/完成/失败和连接状态 |
| `src/stores/chat.svelte.js` | 对话、当前对话和消息容器；执行行状态机留到4.5 |
| `src/stores/editor.svelte.js` | 共用当前素材/工具/对比状态，新增版本、比例和属性容器 |
| `src/stores/prefs.svelte.js` | 对话栏、树宽、缩略图档位和视图偏好 |
| `src/studio.svelte.js` | refresh可接收服务快照并在mock时跳过旧缩略图请求；保留旧草稿恢复逻辑 |
| `src/api.js` | 旧组件请求委托给services/http，原有调用接口保持 |
| `src/main.js` | 会话从services读取，环境开关在真实入口生效 |
| `src/App.svelte` | 使用新的启动/重试/销毁服务与项目store，不直接调用新接口 |
| `.env.example` | 说明VITE_FRONTEND_DATA_MODE开关，默认real |
| `scripts/test-target-ratio.mjs` | target/SelectionContext与比例单元测试 |
| `scripts/test-frontend-services.mjs` | 适配器、未知元数据、SSE、授权/写入和Worker RPC单元测试 |
| `scripts/verify-frontend-data.mjs` | 浏览器验证八个store、对象身份、rAF、单项事件及真实Worker/缓存 |
| `scripts/verify-frontend-mode.mjs` | 验证环境变量选择的mock生产构建不发API请求 |
| `docs/frontend-spec.md` | 仅将4.2改成已完成 |
| `docs/frontend-decisions.md` | 记录数据归属、未知值、上下文与mock边界 |
| `docs/frontend-inventory.md` | 更新阶段状态，不改4.0历史盘点事实 |
| `docs/frontend-4.2.md` | 本任务的逐文件说明、验收、测试和后续确认 |

## 验收结果

- ✓ target覆盖@引用 > 当前选中 > 编辑器当前图 > 当前筛选，去重且保持输入顺序，不修改输入。
- ✓ 超过50张仍返回全部目标；按最新规则不添加第四种ConfirmRow。
- ✓ 非编辑页面不会误用缓存的editorAssetId；fileNames只含名称；序列化只输出SelectionContext的规定字段。
- ✓ ratio覆盖五种比例×两个方向、默认方向重置、实际标签、⇄交换及1:1无动作；换算像素误差不超过1px；留空长边不放大，显式长边按参数处理。
- ✓ 八个store已建立；项目/素材/选择/编辑和旧模块使用同一份数据，未把资产数组复制为另一份可写状态。
- ✓ 选择变化通过rAF合并；浏览器本轮样本约22ms更新，低于100ms。本结果不替代后续5000张素材性能验收。
- ✓ 文件句柄恢复不主动弹授权；用户明确操作才请求权限；写入前复查，生成versions/trash/cache目录，重复文件使用“名称 (2).扩展名”。
- ✓ Worker内SHA-256匹配已知摘要；真实缩略图尺寸160×80；重建Worker后IndexedDB命中缓存。
- ✓ 环境变量real/mock分离，错误值直接报错；mock构建从真实应用入口启动且无任何API请求。
- ✓ 真实模式只调用已存在的bootstrap/model_catalog/hybrid_jobs/session等能力；未知队列位置/开始时间/配额不伪造。
- ✓ 未获批准的operation_execute/chat_execute不发送请求，只返回明确的缺能力错误；没有改P0–P3业务逻辑。

## 测试

- target/ratio/SelectionContext：14/14。
- services/文件授权/事件/Worker RPC/元数据：7/7。
- 4.1单元测试：4/4；coverflow单元测试：17/17；历史P1/P2前端回归各8/8，总计58项通过。
- Svelte递归编译检查通过，含8个store和工作区协调模块。
- real和mock分别构建到 `output/playwright/frontend-build` / `frontend-build-mock`，不覆盖受控dist。
- 浏览器数据报告：`output/playwright/frontend-data/report.json`；环境开关报告：`output/playwright/frontend-data/mode-report.json`。
- 4.1的7组浏览器回归继续通过，17张截图更新，pageErrors=[]。重试按钮曾把点击事件误传作适配器参数，已修正并复验错误恢复。
- 数据验收脚本按Vite实际加载URL导入模块，避免测试自身使用不同HMR URL创建第二个store实例；源码使用现有Svelte状态模块导入约定。
- 4.1已批准的Rust页面路由及55项后端回归仍为通过状态；本任务没有继续修改后端。

## 自行做的决定

- 既有studio继续持有供原模块读取的数据；新store提供职责入口及派生视图，后续组件从store访问。旧资产和草稿恢复逻辑不重写。
- real默认开启，mock只在services或测试夹具中。没有服务端证据的字段保持unknown/null，不把null当0。
- Mock执行只产生明确标记的模拟任务；不生成或声称存在处理后的真实图片。进度可由测试/开发入口显式推进。
- 新操作出站时立即获取最新SelectionContext快照，UI显示用rAF合并，不会因等待一帧而提交旧选择。
- 比例选择/交换纯函数先交付，真正生成版本与批量撤销仍在4.8/4.9结合确认后的接口完成。
- 详见 `frontend-decisions.md` D015–D018。

## 需要确认的事项

4.2无新增依赖或后端变更需要确认。后续用户已确认保留原模块监听，shortcuts.ts统一管理外部按键并让原版优先；4.3已据此完成，不改原版按键。

## 下一步

4.3快捷键系统。之后按4.4→4.5–4.8→4.9→4.10→4.11推进。后端操作/聊天/配额等草案仍未获批准，不把此次页面路由批准扩大为其他接口批准。
