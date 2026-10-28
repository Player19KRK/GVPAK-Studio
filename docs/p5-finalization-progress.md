# P5 最终联调进展

更新：2026-10-28。4.11 已完成；当前正式应用已更新，可通过原“一键启动.cmd”测试。P5.1/P5.5 的真实 Windows 外部目录证据未完成，因此不宣称全部验收交付。

| 任务 | 当前结论 | 证据与边界 |
| --- | --- | --- |
| 5.1 首页一句话到文件夹 | 待验收 | 真实 DeepSeek 规划→OpenCV 抠图/白底/4:5→自动写回、目录筛选及无点击链路已在 OPFS 通过；Windows 原生目录写回尚未实测。见 output/playwright/pipeline/home-natural-receipt.json。 |
| 5.2 本地斜杠比例 | 通过 | output/playwright/p5/ratio.json：真实输出 225×400，DeepSeek 请求 0。 |
| 5.3 50 张撤销 | 通过 | output/playwright/local-plan/batch-50-undo-receipt.json：50 个不同 PNG、50 个输出及新版本，一次撤销，原图逐字节一致。 |
| 5.4 主机优先及排位 | 通过 | output/playwright/p5/queue.json：独立数据根、真实 TLS 同事登录，主机排位 0/同事排位 1，主机先开始，两者完成。抽屉排位展示与推送有 UI 回归。覆盖本版标准任务，不宣称改写旧 P0–P3 调度或验证了两台物理设备。 |
| 5.5 关闭重开写回 | 待验收 | output/playwright/local-plan/live-writeback-receipt.json：实际关闭运行页面后，OPFS 恢复句柄、自动写回、重复重开不重复下载及撤销通过。真实 Windows 目录的授权恢复与写回尚未实测。 |
| 5.6 原版 coverflow | 通过 | output/playwright/p5/coverflow/receipt.json：完整应用九组交互、状态角标及 URL 检查；18 项纯回归；授权只读参数之外的原版及辅助源码指纹一致。列表来源的宿主专项亦通过。 |
| 5.7 主题与尺寸 | 通过 | output/playwright/theme-final/receipt.json 的 32 张主要页面，加 theme-supplement/receipt.json 的 28 张附属页面截图；明暗×1280/1920，无溢出、无名控件或缺失图标提示。登录输入对比度≥4.5:1。 |

## 最后一个阻塞条件
原生 computer-use 工具读取隔离 Chrome 时报告：无法可靠确认当前浏览器 URL，终止本轮原生操作。已停止原生输入，未改用其他方式绕过该限制。

已准备隔离目录 .tools/p5-native-20261028，只有测试图片：新品/a.png、新品/b.png、outside.png。测试地址 http://127.0.0.1:8796/library 连接隔离后端 8799，不是正式素材库。2026-10-28 已请求一次协助，用户回复“暂时无法操作”，因此不重复索要操作、不虚构完成证据。

未来完成 5.1：选择此目录，清空选中及 @ 引用，首页发送“把 新品/ 的图抠图做 4:5 白底”，发送后不点击；核对两个经 SHA 校验的 4:5 白底文件、outside.png 未处理、原始三个 PNG 字节保持。自然语言测试会使用已配置 DeepSeek 文本接口，不涉及图像生成。

未来完成 5.5：在真实授权目录提交本地任务，运行中关闭页面，再打开；核对自动写回、权限恢复和再次重开不重复结果。上述两项完成前保持清单未勾选。

## 已部署版本及其他验证
- 正式入口 http://127.0.0.1:8787/，根目录“一键启动.cmd”；原启动链冷启动通过，OpenCV 4.8.0 可用，新品模块配置可读取。Gemini/DeepSeek 沿用在线服务配置，本地像素处理使用 Rust/OpenCV。
- 真实后端自动摆正：输入主体倾斜 11.793°，实际返回像素 0.0°；新增一个版本、原图 SHA 保持，供应商请求 0。见 output/playwright/p5/opencv-deskew.json。
- 前端 100 项纯测试 + 40 项浏览器条件回归、coverflow 18 项通过；后台二进制沿用 126 项已通过的发布构建，本次未改后台逻辑。
- 区域/配置重试、键盘、新品七组、mock 零 API 请求及 real/mock 构建通过。新品 ZIP 在默认执行隔离环境被取消，提升本机测试权限后通过，产品下载逻辑未修改。
- 正式包性能样本：5000 素材、首屏 834.1ms、60.00fps、全选 16.6ms、斜杠 42.3ms、推送最大 34.2ms、重开缩略图下载 0。本机夹具包含真实 Worker、编码和 IndexedDB，不代表其他设备的外部网络耗时。
- 冷启动前后 4 项目/63 素材的 ID、原图 SHA 和版本数不变，证书及后端二进制不变。当前全部 22 个构建文件的 HTTP/磁盘字节一致，页面回退及静态/API 404 正常，团队 TLS 验证通过。见 output/playwright/finalization/deployment.json、http.json。

4.11 全部改动路径与验收见 frontend-4.11-final.md；历史子项记录保留，不用本记录掩盖真实目录的缺失验收。
