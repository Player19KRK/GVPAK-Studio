# 4.6 首页验收

## 文件
- src/components/HomePage.svelte：居中输入、四个快捷指令、最近项目/任务；发送后导航到素材库并保留对话。
- src/lib/home.ts：对象优先级、明确目录请求、仅文件名的SelectionContext。
- src/services/chat-files.js：导入当前项目、按导入/日期归档、变为@引用，重复素材保留已有目录。
- backend/src/frontend_assets.rs、core.rs、frontend_api.rs：用户批准的assets_metadata，整批验证后原子提交归档元数据，不改原图。
- backend/src/frontend_ops.rs、frontend_chat.rs：用户批准的generate操作与工具定义，数量/费用门槛、请求/来源/报价绑定的确认令牌、方案缓存、幂等与不明结果恢复保护。
- backend/src/new_product/google.rs、mod.rs：复用原Google网络与图像编解码，单参考图场景请求；没有新依赖或第二条生图服务。
- src/components/ChatPanel.svelte、ExecRow.svelte、stores/chat.svelte.js：结果、停止后返回结果、跨项目查看与新编辑保留提示。
- src/components/CompleteStudio.svelte、FoundationPage.svelte、styles/base.css：首页与既有完整应用/壳接线、路由与对话高度。

## 验收
- ✓ 首页发送后进入/library，相关素材已选中，同一个对话接着显示执行行。
- ✓ 拖入/选择图片上传后真实保存导入/YYYY-MM-DD元数据，自动生成@引用；刷新后保留。
- ✓ 统一比例快捷指令直接执行，不发DeepSeek；场景快捷指令使用固定模板直接走同一对话与后台队列，不额外调用DeepSeek；自由文字生成要求才走工具规划。
- ✓ 5张场景任务返回一行确认；继续时复用相同requestId/token和缓存方案，不重复规划；取消不提交。
- ✓ 场景参数只有规定比例/方向/分辨率，每张选中参考图生成一个版本；后续本地处理可接在同一任务内。
- ✓ 标记提交发生在网络请求前，重启后未知结果不自动重发；保存过的生成版本恢复时不覆盖更新的编辑。
- ✓ 旧新品套图入口、并发/异步能力保留，回归通过。

## 测试证据
- scripts/verify-home.mjs：4组首页/元数据/导航/对话检查。
- scripts/verify-scene-chat.mjs：4组场景门槛/确认/结果检查，明确为模拟数据。
- scripts/verify-home-library-live.mjs：隔离真实后端首页导入→比例→素材库→编辑器→新版本→撤销/刷新，6组检查，原图哈希不变。
- scripts/test-home.mjs：来源与目录解析；backend frontend_assets测试验证元数据原子性与原图字节保留。
- backend frontend_ops/frontend_api/Google夹具：单参考图真实编码/解码、请求与来源确认绑定、计划缓存、幂等、不明结果禁止自动重发、新编辑保护。
- 单元测试构建阻止新流程读取真实云端凭据；早期测试未调用付费服务；后续真实连通测试见下节，不能据此声称用户真实包装的批量一致性已通过。

## Windows持久化修复
任务快照与原子写入并发时曾出现OS错误5，并且快照枚举误把完整临时文件当作任务记录。现只读正式UUID.json，保留完整临时文件并对Windows 5/32/33共享冲突短时重试，总等待上限约315ms。原失败的缓存场景流程和人工占用文件回归均通过；不删除原文件、不改任务优先规则或账户规则。临时诊断已移除。

## 未完成边界
P5整体尚未验收，真实云端输出、真实配额计量、旧队列统一调度仍待后续工作。4.9 coverflow列表参数许可尚未收到，不修改原版内部代码。

补充回归：新版新品拖入原先会冒泡给旧导入器并产生两次上传，已通过独立事件归属修复；verify-new-product.mjs验证一次上传、保留当前模式、后续确认和整套结果下载。

## 真实云端连通补验
通过独立dev数据目录与现有凭据，提交1张自制容器示意图的1K场景请求，Google真实返回1024×1024成图并存为新版本，原始输入哈希不变。再用该测试图调用Gemini3.8Flash，真实返回1个产品及结构化澄清问题。未使用用户照片。首次test凭据命名空间缺密钥以及一次本机启动就绪前连接失败均发生在云端提交前；没有对结果不明的请求自动重发。
证据：.tools/cloud-smoke-dev/output.png、receipt.json、analysis-receipt.json。此补验覆盖真实账号连通、结构化响应与产物保存，不代表用户包装/真人/爆炸图质量全部验收。
