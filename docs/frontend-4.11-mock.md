# 4.11 完整应用mock隔离

2026-10-28，real与mock两种独立构建均已验证。正式应用继续real；4.11/P5整体仍未勾选。

## 改动文件
- src/services/adapters/mock-workspace.ts：同一内存工作区的项目回收/恢复、真实File字节导入、SHA去重、原图读取和兼容命令；未实现能力明确拒绝。
- src/services/adapters/mock.ts：共享工作区，元数据批次原子更新；测试进度演示与实际应用区分。
- src/services/api.ts：兼容服务入口；应用mock禁用人工任务推进与真实处理。
- src/services/http.ts：mock命令分派；原生HTTP请求在mock构建直接拒绝。
- src/services/account.ts：登录、登出、下载列表、反馈的统一HTTP服务，保留真实模式请求形状和响应语义。
- src/api.js：兼容客户端按环境接入mock上传和文件选择，不走真实上传。
- src/studio.svelte.js：mock初始化不创建原生EventSource。
- src/services/new-product.ts：新品素材与主工作区共用，mock不伪造识别内容、任务单生成或图片。
- src/services/library.js：mock网格显示实际导入原图的缩略图。
- src/services/settings.ts：应用mock拒绝真实密钥保存与证书更换；独立测试构造器保留显式状态演示。
- src/stores/project.svelte.js：mock不绑定真实输出授权，不创建.gvpak或输出目录。
- src/components/CompleteStudio.svelte：模拟构建常驻模式与刷新重置说明。
- src/components/NewProductMode.svelte、WorkspaceSettings.svelte：明确模拟能力与本机工具状态。
- src/components/Login.svelte、TeamBar.svelte、Feedback.svelte：移除组件原生fetch，调用服务。
- scripts/new-product-trial-server.mjs：mock静态验收可禁止所有API转发，默认真实验收代理保持原行为。
- scripts/test-mock-workspace.mjs：6项共享状态、实际字节、去重、不可用能力、敏感操作和元数据原子性测试。
- scripts/test-account-service.mjs：2项账号/反馈/下载传输契约回归。
- scripts/verify-complete-mock.mjs：生产mock构建完整应用，无后台代理，所有API尝试都会阻断并判失败。
- scripts/check-component-transport.mjs：组件禁止原生HTTP和EventSource的源码回归检查。
- docs/frontend-decisions.md：D064/D065；frontend-4.11-audit.md：同步证据。

## 验收
✓ 完整应用启动：首页、素材库、编辑器、新品、设置可用，API/EventSource请求0、页面错误0。
✓ 真实图片字节在浏览器内存导入、SHA去重，主素材库/新品/编辑器共享，不填充假成图。
✓ 新品手动用途及彩色套框可交互，识别/生成明确未启用；处理请求明确失败，不伪造后台完成。
✓ 文件夹导入未创建.gvpak、输出目录或绑定输出权限；用真实OPFS目录句柄读回目录项验证。
✓ mock证书、密钥及反馈不访问真实后台，也不显示真实提交成功。
✓ 刷新清空模拟数据，模式提示与行为一致；不将模拟内容写入正式工作区。
✓ 批次元数据中有无效素材时拒绝整批，工作区不留下前半批修改。
✓ 正式构建继续真实模式，有实际bootstrap读取，无mock标识；4项目/63素材ID、原图SHA、版本数保持。

## 模式与边界
VITE_FRONTEND_DATA_MODE在Vite构建/开发启动时生效，值为real或mock；一键启动使用已经构建的正式dist，不在启动时重新打包。此次real产物在dist，mock隔离产物在.tools/mock-app/dist，静态验证地址http://127.0.0.1:8794/，该静态进程明确不转发API。

mock支持内存项目与原图界面交互，不代替真实处理/付费供应商。独立createApi('mock')测试构造器可显式演示运行/取消；应用getApi的mock实例禁止该模拟执行，不生成假像素、输出文件、识别结论或敏感资源。

## 测试与部署
前端已有93+37项通过；另新增账号契约2项通过，共132项。真实/模拟两种发布构建、Svelte编译、反馈契约、组件传输边界和coverflow冻结检查通过。真实模式项目回执、区域错误、配置读取、单图生成及新品回归通过，无付费调用。
证据：output/mock-pure-tests.log、mock-regressions.log；output/playwright/mock-isolation/receipt.json、deployment.json、output/playwright/mock-isolation.png；元数据原子性的修复前失败记录output/mock-metadata-red.log。

正式http://127.0.0.1:8787/，OpenCV4.8.0一键启动检查通过；旧入口备份.tools/deployment-backups/20261028-095838-p4-mock-isolation/。本次只更新静态前端，未重启后台，无后端逻辑或依赖变化。

## 后续
旧导出页底层export_run/export_preview的SelectionContext服务适配、其他K细节、附属视图主题及真实设备/跨账号队列等P5仍需完成。本子项不是全部交付证明。
