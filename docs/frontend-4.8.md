# 4.8 RatioBar 验收

## 文件
- src/components/RatioBar.svelte：唯一共用比例控件，显示实际横竖比，1:1禁止交换；提交及对应任务执行期间禁止重复点击。
- src/components/AssetLibrary.svelte：右键与批量菜单共用该控件。
- src/App.svelte、components/CompleteStudio.svelte：编辑器40px工具条接入；快捷键由shortcuts.ts既有入口分发。
- src/services/commands.js：沿用ratio.ts纯函数与同一执行服务；Ctrl+Z选最新可撤销批次。
- scripts/verify-home-library-live.mjs：真实隔离数据端到端证据；scripts/verify-library.mjs：比例文字、交换、禁用和菜单/撤销检查。

## 验收
- ✓ 点击比例恢复默认方向；方向交换后3:2变2:3，16:9变9:16。
- ✓ 1:1交换按钮禁用。
- ✓ 不显示规格禁止的名称或额外比例。
- ✓ 真实后端：快捷比例4:5、编辑器16:9、交换9:16各生成一个版本；共3个版本。Ctrl+Z恢复前一16:9版本，原图哈希保持一致。
- ✓ 刷新后日期目录、历史版本保留；没有DeepSeek或生成请求。
- ✓ ratio纯函数、执行状态机与相关快捷键冲突表单测已有覆盖；浏览器实测通过。

当前仅将4.8勾选。4.6的场景生成扩展和4.9的coverflow列表参数仍待用户确认；不将审批未完成项记为成功。
