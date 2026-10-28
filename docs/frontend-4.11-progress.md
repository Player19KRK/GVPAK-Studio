# 4.11 进行中：单图生成与执行行收拢

4.11尚未完成，不勾选。已完成本轮子项并部署到完整应用。

## 改动文件
- src/components/Generate.svelte：纯文本/参考图统一任务提交；五种比例加交换；分辨率无障碍名称；移除Review与成功toast入口，历史直接切换编辑版本。
- src/services/single-generation.js：生成对象投影、空文本上下文、服务层项目创建和凭据状态。
- src/services/adapters/real.ts、mock.ts：新增项目/凭据服务适配；mock文本任务明确计1张，不生成假图片。
- src/stores/chat.svelte.js：空文本任务、SelectionContext响应式安全序列化、重试保留显式操作、Esc取消待确认。
- src/stores/assets.svelte.js、src/services/workspace.svelte.js：完整结果事件可插入所属项目，状态事件不造占位素材。
- src/studio.svelte.js、src/components/CompleteStudio.svelte：按需显示生成页对话，状态仍由根订阅保持。
- src/components/AppShell.svelte、src/App.svelte：备用入口的生成对话与Ctrl+K/J接线。
- src/components/ExecRow.svelte：纯文本对象和通用生成标签。
- backend/src/frontend_ops.rs：文本/模型参数、空上下文校验、真实价格确认、持久像素恢复、统一版本/回收撤销及费用元数据。
- backend/src/core.rs：仅新增create_generated_asset方法；收到真实像素后建立新素材，既有导入/保存逻辑不改。
- config/models.json：补齐Nano 4:5/5:4能力，参考Google官方ImageConfig；报价未更新。
- scripts/studio-test-harness.mjs、test-p2-regressions.mjs：旧“approved:true直接generate”测试改为统一服务边界，保留草稿/粘贴/最新文字/文字不引用图片回归。
- scripts/test-new-asset-events.mjs：跨项目事件归属、仅完整素材可插入、无占位状态素材。
- scripts/verify-single-generation.mjs：生产构建fixture浏览器覆盖Poe参考图、比例交换、确认、文本、取消重试与Esc。
- scripts/verify-single-generation-live.mjs：生产构建+真实后端，仅验证费用确认/取消/参数保持，不批准供应商调用。

## 本轮验收
✓ 单图流程不再发送approved:true或原始assetId/edit处理上下文；使用SelectionContext。
✓ 纯文本上下文不含图片ID/文件名，先有任务，收到实际像素才有素材。
✓ 参考图直接新增可撤销版本；文本新素材可undo/redo，保留原图。
✓ 现有Poe/Google模型未被删除；未知费用进入ConfirmRow。
✓ 已返回像素先保存再采用；重启不重复供应商请求或创建素材，未知结果标为uncertain。
✓ 确认沿用请求ID；取消不提交；重试保留模型/文字/方向/分辨率，修复Svelte Proxy不能structuredClone的真实故障。
✓ 真实4K未知费用请求只进入确认，取消/重试没有素材增加，无pageerror。
✗ 4.11全任务仍未完成：Assistant旧流程、全区域四态、其余成功toast/Confirm/Review、全应用明暗/性能/mock检查，以及P5文件夹写回、团队保留和调度仍按audit继续处理。

## 测试与实际证据
- Rust98项通过，包含文本缓存像素恢复、无结果不重发、回收/重做、空文本费用数量为1、原图保护及既有回归。
- target/ratio/shortcuts/execution/services39项通过；P1与P2/export共16项通过；新素材事件1项通过。
- fixture单图浏览器6组及真实后端单图5组检查通过，取消后的重试与Esc覆盖。新品生产构建7组回归通过。
- Svelte编译和生产构建通过。coverflow授权参数外基线不变，辅助6文件指纹不变。
- output/playwright/single-generation/receipt.json、live-receipt.json；output/single-generation-backend-tests.log、single-generation-js-tests.log、single-generation-p1-tests.log、single-generation-p2-tests.log。
- 本轮付费供应商提交0。供应商成图恢复使用明确的合成缓存像素fixture；没有据此声称真实Poe/文本供应商生成质量已验证。
- 非方比例参数参考：https://ai.google.dev/api/generate-content 。未额外生成非方图片。

## 部署
bin/dist已更新，start.ps1启动检查OpenCV4.8.0正常。正式数据4项目/63素材的ID、SHA256和版本数一致，正式证书哈希不变；exe/index与验证构建哈希一致。

自行决定：D045–D047；无新依赖。
需要确认：无。
下一步：迁移Assistant旧规划/预览/执行链路；继续4.11与P5，不缩小目标。


## 后续进展
本地手动步骤、统一输出和整批撤销已完成并部署，见 [frontend-4.11-local-plan.md](frontend-4.11-local-plan.md)。自然语言混合/Batch分支等仍待处理，4.11保持进行中。
