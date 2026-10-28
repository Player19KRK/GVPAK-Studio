# 4.11 性能实测与修正

2026-10-28，已部署正式前端；4.11及P5整体仍未勾选。

## 修正的问题
1. 原编码缓存只在下载后按hash命中，刷新仍向后台取图32次。新增本机源版本索引，网格和旧宿主共用；刷新时不再下载源缩略图，编码Worker从已有hash/size缓存返回。
2. 5000张全选实测同步643ms。Profiler显示主要耗时在每个id读取assets.availableIds时重复执行Svelte派生依赖检查；缓存一次集合后仍逐个校验id，同步降至12–17ms。引用和@候选列表修正同样的读取模式。

## 改动文件
- src/services/thumbnail-cache.ts：账号/工作区/项目/素材/版本源索引、实际内容哈希、并发合并、失效及缓存不可用时原路径；异步身份使用快照。
- src/services/library.js：网格复用源字节，编码仍在既有Worker；解码失败清源索引，允许有效重试。
- src/studio.svelte.js：旧宿主缩略图复用同一源缓存，保留force及版本采用前验证。
- src/stores/selection.svelte.js：选择和引用一次读取可用集合。
- src/components/Composer.svelte：候选筛选一次读取可用集合，保持原筛选结果与20条上限。
- scripts/test-thumbnail-cache.mjs：6项缓存回归，覆盖持久重开、账号/版本隔离、并发、force/失效、缓存失败、异步源身份快照。
- scripts/test-selection-performance.mjs：批量选择/引用原来读取集合5000次的失败回归，修复后每操作一次。
- scripts/studio-test-harness.mjs：注入独立缓存边界，保留原状态/编辑测试；真实缓存另有单元及浏览器验证。
- scripts/verify-frontend-performance.mjs：生产构建、5000条素材、真实缩略图/哈希Worker与IndexedDB，采集首屏、滚动、选择、全选、斜杠、推送与重开缓存，可选Profiler/Tracing。
- scripts/check-frontend-performance.mjs：验证记录的发布样本对应T预算。
- docs/frontend-decisions.md：D059/D060；frontend-4.11-audit.md：同步证据及剩余范围。

## 最终发布样本
| 指标 | 实测 | 规格 |
|---|---:|---:|
| 5000素材首屏缩略图 | 789.6ms | <1000ms |
| 快速滚动 | 60.00fps，P95 16.7ms | 60fps |
| 挂载素材卡片 | 首屏21，最高33 | 虚拟列表 |
| 单张选中到SelectionContext | 12.8ms | <100ms |
| 5000张全选到SelectionContext | 16.2ms | <100ms |
| /白底进入running | 36.3ms，DeepSeek请求0 | <300ms |
| 推送消息到界面绘制 | 最大34ms | <200ms |
| 刷新后台缩略图请求 | 0 | 不重新生成 |
| 刷新编码Worker | 24次缓存命中，0次编码未命中 | hash缓存 |

滚动样本240帧、每帧190px，完整数据集5000条，访问生成690个不同像素缩略图；不把相同图片重复填5000格伪装成大图库。进度采样从EventSource收到的MessageEvent分派开始，经原JSON解析/事件总线/任务状态到两次requestAnimationFrame后的界面，不包括网络传输。接口为本机隔离夹具，不是实际照片或外部服务延迟验收。

此前相同压力采样有59.75fps及1次33ms帧；另一次Trace记录20ms MajorGC，不能据此把其他帧全部归因于GC。原始记录均保留，未覆盖为单次绿灯；真实照片、同事设备和长时间性能继续在P5复核。本次证据不代表每台设备永无掉帧。

## 验收与测试
✓ 对应T的发布样本预算检查通过。
✓ 前端纯测试79/79、浏览器条件31/31，共110项；Svelte检查及发布构建通过。
✓ 原有区域重试、编辑器撤销和新品7组回归通过；coverflow冻结检查通过。
✓ 无后端接口/业务修改，无新依赖，无付费调用。
✓ 正式入口构建一致，一键启动OpenCV4.8.0可用；4项目/63素材ID、原图SHA、版本数保持。

证据：output/playwright/performance/before.json、after.json、profile.json、final.json、frame-audit.json、trace.json、release.json；selection.cpuprofile及scroll-trace.json；output/selection-performance-red.log；performance-pure-tests.log、performance-regressions.log。正式部署deployment.json。

入口http://127.0.0.1:8787/，继续一键启动.cmd；本次只更新静态前端，未重启后台。旧index备份.tools/deployment-backups/20261028-072835-p4-performance/。
