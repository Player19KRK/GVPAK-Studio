# 4.11 历史盘点

> 此文保留收尾前的历史缺口。当前状态以 frontend-4.11-final.md 和 p5-finalization-progress.md 为准；不要将下列历史“待处理”当作当前状态。

4.10后按当前源码核对，以下未完成，不勾选4.11/P5：

- 已完成：Generate.svelte单图流程已迁入统一执行行，支持纯文本/当前参考图、原Poe/Google模型、五比例+交换、成本确认及取消重试；详见frontend-4.11-progress.md。旧接口保留兼容，单图页不再调用旧generate或打开Review。
- 本地手动步骤已迁移到SelectionContext/local_plan、直接执行、新版本和校验写回；预览/sample应用已从该分支移除。Assistant自然语言混合管线已迁入统一执行行，保留实时/Batch和既有模型能力，详见frontend-4.11-pipeline.md。
- 完整应用成功toast及旧Confirm/Review已移除；项目回收先保存草稿，执行行支持刷新后恢复，导出明细移入对话。本机失败回执禁止进入云重试，见frontend-4.11-receipts.md。导出页已迁入SelectionContext预览及标准导出任务，保留尺寸/质量并校验写回，见frontend-4.11-export-contract.md。
- 完整应用8个主要视图已补齐外层明暗主题并完成1280/1920截图矩阵，见frontend-4.11-theme.md；coverflow保持原版计算颜色。旧项目视图、团队登录/下载等附属状态仍需纳入最终P5逐页复核。
- 工作区、任务、缩略图、设置、导出效果的加载/失败/重试及筛选空状态已补齐并通过故障注入浏览器回归，见frontend-4.11-regions.md。普通生图/自然语言配置读取也已补齐加载、错误重试、未配置和可用状态，模型切换拒绝迟到响应，见frontend-4.11-configuration.md。预设等其余区域四状态和键盘全链路仍需验收。T基准已测首屏/选中/斜杠/推送/缓存，修复全选与重复取图；快速滚动约59.75–60fps，曾有偶发长帧，另有GC记录，见frontend-4.11-performance.md，仍保留真实数据及最终P5性能复核。
- 完整应用mock环境隔离已补齐，启动、导入、导航、用途分配和敏感操作验证零API请求；账号/反馈/下载原生HTTP收进services，见frontend-4.11-mock.md。导出页处理契约已迁移；其他服务适配继续按原规格审核。
- 普通任务已实现校验写回和重开恢复，Chromium OPFS真实目录句柄+真实后端验证通过；首页自然语言到校验落盘已用真实DeepSeek、本地OpenCV及OPFS目录句柄验证；24h保留有单元测试。外部Windows目录和跨设备保留仍待最终P5验收。
- 团队保留策略已加入operation/new-product活跃引用、规划期及完成后24h保护，仍须跨账号联调；调度须证明主机优先与真实队列位置，避免仅验证标准任务队列。
- 50张字节级undo已真实验证，且中断批次已恢复；其他P5项与全部页面四种主题/宽度组合仍需逐项验收。HTTP初始响应与SSE进度交错的单调状态已通过真实后端延迟响应测试；编辑器草稿优先撤销、恢复草稿撤销/重做、输入框原生撤销已补齐回归，详见frontend-4.11-history.md。

当前可复用证据：frontend-4.9.md、frontend-4.10.md，以及output/playwright下各receipt；这些不替代以上缺失验收。
