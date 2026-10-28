# 4.11 最终验收记录

2026-10-28。沿用 Svelte 5/Vite、既有 UI 和后端；无新依赖。本记录与各 4.11 子项文档共同构成完整记录，P5 的真实 Windows 目录验收单独保留，不以 OPFS 证据替代。

## 任务编号与名称
4.11 空/错误状态、无障碍、性能检查。

## 改动的文件
此前的处理管线、导出、mock、缓存和区域状态改动见同目录 `frontend-4.11-*.md`。本次最终收尾文件如下：

| 文件 | 改动 |
| --- | --- |
| src/components/Assistant.svelte | 本地预设读取区显示加载、错误重试、空结果和正常列表。 |
| src/components/Canvas.svelte | 为四个画布图标补齐与可访问名称一致的提示。 |
| src/components/Composer.svelte | 隐藏文件输入退出 Tab 顺序，保留可见添加图片按钮。 |
| src/components/AssetLibrary.svelte | 隐藏文件输入退出 Tab 顺序，防止键盘撤销流程陷入文件控件。 |
| src/components/NewProductMode.svelte | 隐藏输入退出 Tab 顺序，配置读取骨架和原地重试，读取未完成时不误报缺少密钥。 |
| src/stores/new-product.svelte.js | 独立管理配置加载、失败、重试和销毁后的迟到响应，不重复建立轮询。 |
| src/components/Library.svelte | 补齐图标提示与无项目时的空状态。 |
| src/components/TeamBar.svelte | 下载列表读完前不宣称为空，错误可以原地重试，关闭按钮有提示。 |
| src/components/Feedback.svelte | 关闭问题面板的提示包含 Esc。 |
| src/styles/complete-app.css | 登录和附属界面跟随主题；确保深色输入文字对比度和键盘焦点、交互反馈。 |
| scripts/test-new-product-read-state.mjs | 配置四状态与销毁后不采用迟到响应的两项测试。 |
| scripts/verify-final-keyboard-presets.mjs | 发布包的键盘选中、发送、整批撤销与预设错误恢复验证。 |
| scripts/verify-final-read-states.mjs | 新品配置及团队下载的加载、失败、重试与空状态验证。 |
| scripts/verify-supplementary-themes.mjs | 项目、反馈、团队下载、登录及空项目的明暗/尺寸截图与名称、提示、输入对比度检查。 |
| scripts/verify-final-accessibility.mjs | 八个主要视图的主题、尺寸、焦点与图标提示检查。 |
| scripts/verify-p5-coverflow.mjs | 完整发布应用中逐项驱动原版交互和外层状态覆盖层。 |
| scripts/verify-p5-ratio-live.mjs | 本地斜杠解析到真实后端像素的 9:16 验证。 |
| scripts/verify-p5-queue-live.py | 隔离主机、真实 TLS 同事账号与排队优先级联调。 |
| scripts/verify-opencv-deskew-live.py | 用真实后端返回的像素测量约 12° 主体倾斜纠正。 |
| docs/frontend-decisions.md | 记录最后的状态、隐藏输入和主题决定。 |
| docs/p5-finalization-progress.md | 汇总真实联调证据及仍未通过的 Windows 目录验收。 |

## 验收结果
- ✓ 工作区、任务列表、素材缩略图、设置、导出效果、普通生图配置、自然语言配置、本地预设、新品配置、团队下载等异步区域区分加载和失败，并提供有效重试；空结果不冒充请求失败。无项目和无素材有明确入口。处理动作继续在原执行行反馈。
- ✓ 图标有可访问名称和提示，隐藏文件输入不占用键盘主流程；键盘可以完成选素材→对话→撤销。
- ✓ 主要及附属界面在浅色/深色、1280/1920 下检查。登录输入对比度至少 4.5:1；有焦点、hover/active 和 reduced-motion 支持。coverflow 按用户冻结例外保留原版样式与时长。
- ✓ 正式发布构建的 5000 素材性能样本：首屏 834.1ms、滚动 60.00fps/P95 16.7ms/无 >25ms 长帧、最多 33 个挂载卡片、单选 12.9ms、全选 16.6ms、斜杠进入运行 42.3ms、推送到绘制最大 34.2ms；重开缩略图下载 0、Worker 编码未命中 0。为隔离 API、真实 Worker/IndexedDB 的本机基准，不宣称所有设备或外部网络都达到此数值。
- ✓ 组件通过 services 访问接口；反馈检查无原生 alert/confirm、成功 toast 或旧 Confirm/Review。coverflow 只读参数之外的原版及辅助源码指纹一致。

## 测试
- 前端纯测试 100/100，浏览器条件回归 40/40，coverflow 18/18；Svelte 检查及 real/mock 构建通过。
- 后台保持已验收的同一发布二进制：126/126；bin 与 backend/target/release 的 SHA-256 一致，本次没有后端逻辑修改。
- 区域/配置故障注入、键盘链路、新品完整七组、mock 零网络隔离、主题及覆盖层验证均通过。新品下载在默认执行隔离环境被取消，按本机权限重跑通过，未修改产品下载逻辑。
- 新品配置读取、普通生成及任务执行回归只使用隔离夹具，本次没有新增付费模型提交。

证据位于 `output/final-*.log`、`output/playwright/finalization/`、`final-read-states/`、`theme-final/`、`theme-supplement/`、`p5/coverflow/`、`performance/delivery.json`。

## 自行做的决定
见 D069–D071。设置读取中不显示未配置；隐藏上传控件由可见按钮操作；登录及下载沿用现有布局与 token。

## 需要确认的事项
无代码或接口审批事项。原生电脑操作的 URL 安全校验阻止了 Windows 外部文件夹测试；用户目前无法协助，P5.1/P5.5 的外部目录证据继续待验收。

## 下一步
使用正式“一键启动.cmd”测试；完成真实 Windows 目录写回及关闭重开验收后，才能勾选 P5 全部并宣布全量验收交付。
