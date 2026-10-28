# 4.10 设置页验收记录

任务编号与名称：4.10 设置页，完成并更新到完整应用。

## 改动的文件
- src/components/Settings.svelte：五组导航、字段自动保存、空白密钥替换框、实际模型目录、账号配额、费用门槛、证书指纹/热更新/下载；本页明暗主题样式。
- src/components/WorkspaceSettings.svelte：保留原有工作区、数据目录、布局、通知、OpenCV和预览设置；目录切换保留明确操作。
- src/components/TeamAdmin.svelte：账号管理通过settings服务适配，变更时刷新配额账号列表。
- src/components/FoundationPage.svelte：替换设置占位页，备用应用入口也使用同一设置页。
- src/components/CompleteStudio.svelte、src/App.svelte：生命周期内读取执行偏好，退出时解除订阅。
- src/components/Generate.svelte：常规生图采用已保存默认模型，保留本次手动模型选择。
- src/components/Composer.svelte：达到配置的筛选数量时提示实际范围，不增加第四种确认。
- src/services/settings.ts：设置、密钥布尔状态与只写接口、账号、证书的real/mock适配；mock不生成真实证书或假像素。
- src/stores/execution-prefs.svelte.js：共享执行门槛、模型默认值和刷新状态。
- src/services/task-monitor.js：终态任务后读取真实账号配额。
- backend/src/frontend_settings.rs：类型化配置、批量校验、字段补丁原子保存、账号隔离累计输出配额。
- backend/src/frontend_api.rs、backend/src/main.rs：仅主机设置读写及执行偏好契约、session实际配额。
- backend/src/frontend_ops.rs：生成/费用/配额确认读取配置，确认签名绑定配置和配额，任务完成推送配额；版本导航、回收和导出不计新增处理张数。
- backend/src/new_product/mod.rs：识别与套图数量/费用门槛读取共享配置。
- backend/src/team.rs：团队设置命令403；公开执行偏好白名单；真实DER指纹、证书轮换和TLS热加载；安装脚本只含公开证书。
- scripts/test-settings.mjs：服务层只写密钥、字段补丁、mock隔离与无网络/假证书测试。
- scripts/verify-settings.mjs：五组界面、自动保存、刷新、合成密钥、模型配额及下载回归。
- scripts/verify-settings-live.mjs：生产构建和真实隔离后端的保存、TLS指纹、证书更新与门槛验证。
- scripts/verify-settings-team-live.mjs：真实临时团队账号验证5个403和/settings回首页；结束停用账号。
- scripts/verify-settings-theme.mjs：1280浅色/1920深色文字对比度、无横向溢出和截图。
- scripts/verify-settings-model.mjs：常规生图读取默认模型，手动模型选择在执行偏好刷新后保持。
- 已有浏览器fixture：补充execution_preferences读取契约，不替代真实联调。

## 验收结果
✓ 设置五组完整，原有工作区功能保留。
✓ 只能主机本机打开；真实已登录团队账号直接请求settings_read、settings_patch、certificate_status、certificate_regenerate、certificate_installer均403。
✓ 团队访问/settings自动回首页，没有发出敏感设置请求。
✓ 字段修改后自动保存、字段旁显示状态，刷新后恢复；不需要全局保存按钮。
✓ 密钥读取只返回布尔状态，替换输入框为空，成功后清空隐藏；未修改任何正式凭据。
✓ 默认模型实际影响常规生图；新品/对话指定模型保持明确，不加入未接入选项。
✓ 20/4/2/50四项配置有效；识别/生成门槛真实拦截在模型提交之前，筛选门槛按D004/D042只提示范围。
✓ 配额来自实际持久任务记录，按账号隔离；留空不限，临近配额进入原有ConfirmRow。
✓ 证书SHA-256与真实TLS握手一致，轮换后新TLS连接立即使用新证书，账号数据保留。
✓ 下载脚本嵌入公开证书，安装目标为当前用户；未自动运行系统信任安装。

## 测试
- Rust完整95项通过，包括设置校验/原子保存、有效门槛、真实配额计数、证书TLS热加载、已登录团队403。
- settings/target-ratio/execution/shortcuts共32项通过。
- 设置fixture浏览器5组检查、真实设置6组检查、真实团队2组检查、模型默认/手选2组检查、编辑器导航4组检查通过，无pageerror。
- 明暗设置截图：1280浅色与1920深色，主体文字对比度分别12.91和14.43，无横向溢出；这不代表P5.7全应用所有页面已通过。
- Svelte编译及生产构建通过；git diff --check通过，只有既有CRLF与Vite重复导入提示。
- coverflow仅授权参数变化，恢复两处参数表达式后与4.0原指纹一致；其余6个冻结文件不变。
- 证据：output/playwright/settings/live-receipt.json、team-receipt.json、theme-receipt.json、output/settings-backend-tests.log、output/settings-js-tests.log。
- 本轮付费供应商提交0。真实门槛测试停在确认行，没有批准发送。

## 自行做的决定
见frontend-decisions.md D042–D044：自动保存、三种确认优先、按本机输出张数统计配额、证书热更新和模型默认值。

## 部署与边界
正式应用bin/dist已更新，通过start.ps1 -NoBrowser启动；OpenCV4.8.0就绪。部署前后4个项目、63张素材的ID、原图SHA256、版本数一致；正式证书哈希不变；exe/index与验证构建一致。只轮换了.tools隔离服务的证书，无系统信任安装。
本页新配置已接入标准operation_execute和新品流程；旧单图生成/Assistant的上下文、门槛和弹窗入口仍需4.11统一审计，不据此宣称整个P0–P3或P5完成。

需要我确认的事项：无。无新依赖。
下一步：4.11空/错误状态、无障碍与全应用遗留入口审计，再完成P5逐条联调。
