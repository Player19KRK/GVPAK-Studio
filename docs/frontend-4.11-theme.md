# 4.11 完整应用主题子项

2026-10-28，已部署到正式完整应用。4.11及P5总体仍保持未勾选。

## 改动文件
- src/styles/complete-app.css：独立外层主题，标题栏/工具栏/表单/任务/项目/编辑面板等旧色值接入tokens；通用交互规则排除coverflow。
- src/main.js：在原样式之后加载完整应用主题。
- src/components/CompleteStudio.svelte：显式complete-app样式作用域。
- src/components/Feedback.svelte：反馈面板采用主题变量，动画120ms。
- src/components/Inspector.svelte：面板展开动画从180ms调整为120ms，减少动画时保持0。
- scripts/verify-complete-themes.mjs：可重跑的真实浏览器截图矩阵、运行时主题切换和交互样式采集，API全部隔离夹具。
- scripts/check-complete-theme-receipt.mjs：核验矩阵数量、溢出、实际颜色、焦点、减少动画和coverflow计算颜色。
- docs/frontend-decisions.md：D056；frontend-4.11-audit.md：更新剩余范围。

## 验收
✓ 首页、素材库、编辑器、普通生图、新品套图、自然语言处理、导出、设置：1280/1920宽度及浅/深主题，共32张截图。
✓ 全部无横向溢出、无页面异常、无未映射API请求。
✓ 系统主题浅→深→浅不刷新即时跟随；正文与表单采用对应颜色。
✓ 键盘焦点2px，hover和active不同，减少动画时transition为0s。
✓ coverflow冻结源码检查通过，1280浅/深计算颜色与边框逐项对照基线不变；不改变图片内容或用户画布背景。
✓ 相关26项状态/编辑回归通过，Svelte检查和生产构建通过。
✓ 正式页面入口与构建哈希一致，一键启动检查OpenCV4.8.0可用；4项目/63素材ID、原图SHA、版本数均保持。

截图和完整计算值：output/playwright/theme-before/、theme-after/receipt.json；最终核验validation.json、正式部署deployment.json。
实际图像为隔离夹具，截图证明布局与主题，不证明生图质量。测试不会调用付费模型或修改用户项目。

## 部署
http://127.0.0.1:8787/，继续使用一键启动.cmd。只更新前端静态文件，保留旧哈希资源、未重启后台。旧index备份：.tools/deployment-backups/20261028-065753-p4-theme/。

## 后续范围
旧项目视图、团队登录/下载等附属状态、各区域四状态、全量无障碍和T性能指标，以及其他P5项继续验收。本次32张主要视图截图不代替完整P5.7，暂不勾选。
