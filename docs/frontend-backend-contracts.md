# 前端联调待确认接口

状态：用户已批准五个接口及 SSE 契约。session_status/task_list/task_stop/operation_execute/chat_execute 已增量实现；现有接口与新品流程保留。配额账本、旧队列统一调度与后续元数据契约仍须分别完成，不宣称 P5 已通过。

## 4.4 会话、任务与实时事件
- `session_status {}` → `{account:{id,name}|null,isHost:boolean,isLocal:boolean,quota:{used:number,limit:number}|null}`。配额只统计本账号；不能读回密钥。
- `task_list {}` → `{tasks:Task[]}`。Task：`{id,type,projectId,accountId,state,targetCount,done,total,queuePosition:number|null,startedAt:number|null,resultAssetIds:string[],error?:string}`。state：queued/running/done/partial/failed/cancelled；queuePosition 是前面任务数，0 为下一项。
- `task_stop {taskId}`：running 停止后续操作，queued 取消；以 taskId 为界，不误停其他任务；响应 `{task:Task}`。
- 沿用 `/api/events` SSE：`session.updated` 携带当前账号真实配额；`task.progress / task.queue / task.done / task.failed` 携带 Task 增量；`asset.updated` 携带 `{projectId,assetId,patch}`。重连后重新读取快照。
- 服务端保证账号隔离、主机优先，排队期间也能响应取消；不能只由前端模拟。

## 4.5–4.8 执行及对话
所有选择输入只使用 SelectionContext：page、mentions、selected、editorAssetId、filter、filteredCount、resolvedTarget、fileNames。fileNames 仅文件名。
- `operation_execute {projectId,context,operations,clientRequestId,confirmToken?}` → `{task:Task,batchId?}` 或 `{confirmation:{stage,message,token}}`。operations 包含 cutout/white/center/fit_ratio/export/trash/undo/redo；fit_ratio 参数为 ratio、orientation、long_edge?、mode；undo/redo 用 batchId 恢复版本指针。输出新版本；原图不覆盖；删除进入回收区。
- `chat_execute {projectId,context,instruction,clientRequestId,confirmToken?}` → 同上。DeepSeek 只接收 instruction、SelectionContext 和工具定义；不再拼接路径、传统尺寸统计上下文。`chat.text`、`chat.plan` 事件携带 requestId、增量文本/方案。
- 确认只用已批准门槛：识别超过20张；生成超过4张或费用上限；配额剩余不足5%。普通本地执行不出预览卡。不得把“填充透明区域”作为语义抠图成功。
- 元数据与导入接口后续按4.7实际缺口细化，未经确认不修改后端。

## coverflow 尚待决定的独立边界
原版 Svelte coverflow 没有列表参数，直接读取当前项目完整 assets()；仅靠薄宿主不能切换选中/筛选/本批显示列表。不篡改项目索引。当前保持源码及行为。4.9需要用户提供有列表接口的原版，或明确许可增加一个只读数据参数（任何内部改动均须用户明确批准）。


## 4.6–4.7 补充待确认：素材归档元数据
为了让首页拖入图片真实归入当前项目“导入/YYYY-MM-DD”，并让素材库文件夹树在刷新和其他客户端保持一致，请求新增一个接口：
- `assets_metadata {projectId,entries:[{assetId,folder?:string,tags?:string[]}]}` → `{project:Project}`。folder仅项目内相对逻辑目录（禁止绝对路径、盘符和..），tags是字符串列表；只改归档元数据，不移动或覆盖原图字节。
- Project.assets增加可选默认字段folderId、relativePath、tags；旧数据按空目录/空标签兼容；bootstrap与asset.updated一并返回。DeepSeek仍只接收SelectionContext，fileNames仍只有文件名。
- 单次entries≤5000，逐项验证归属，整批验证后原子更新；账号只能修改自己的工作区。前端通过services封装，mock环境模拟，不加依赖。
状态：用户已批准，接口及旧素材兼容字段已实现，正在做4.6验证。


## 4.6 补充待确认：对话中的场景生图
首页规格包含“生成场景图”快捷指令。已批准的operation_execute工具枚举只有本地操作，还缺少场景生成操作。请求在该现有接口新增一种操作（不新建依赖）：
- `{kind:"generate",prompt:string,ratio:"1:1"|"3:2"|"4:3"|"4:5"|"16:9",orientation:"portrait"|"landscape",resolution:"1K"|"2K"|"4K"}`。引用图只来自同一个SelectionContext.resolvedTarget.ids；每个对象生成一张，默认1K/1:1；模型沿用已配置的Nano Banana 2.1。
- 复用现有Google请求和输出保存能力，生成新版本；超过4张或估价超过已配置上限、配额不足5%时返回对话内ConfirmRow，确认token绑定请求。取消仅停止新增提交；已提交任务的结果保留；不自动重发结果不明的付费请求。
- 同一个执行队列/任务抽屉/执行行显示进度与结果；DeepSeek工具定义增加generate，仍只传SelectionContext。
状态：用户已批准，generate操作、确认令牌、方案缓存及恢复保护已实现；测试使用本机provider夹具，不把这些测试当作真实云端成图质量证明。


## 用户授权更新
用户已明确批准 coverflow 仅增加可选只读列表参数，默认原列表，其余交互/样式/辅助源码不改。随后用户要求“继续任务，不需要我确认”，后续为完成原清单所必需的接口细化按决策文档记录并直接实施，继续遵守原图保护、既有技术栈、无新增依赖和coverflow最小授权边界。


## 4.9 编辑器细化（按用户“不需要我确认”授权直接实施）
operation_execute新增独立commit_edit（edit沿用既有Edit结构，保存一次）、mask（mode:add|erase，radius为短边归一化半径0–0.5，points为最多4096个[0..1,0..1]坐标）、background（color:RGB）、deskew（OpenCV最大45度）。fit_ratio增加可选background:RGB。全部复用SelectionContext、幂等任务ID、真实进度和撤销快照，原图保留；不修改旧命令语义。task视图增加assetIds便于编辑区锁定当前处理中素材。颜色/掩膜参数写入任务前验证。


## 4.10 设置、执行偏好与证书（沿用D036授权，无需逐项确认）
仅主机接口：settings_read → {analysisCount,generationCount,costCny,filterCount,imageModel,quotas,usage}；settings_patch {patch} → 已保存配置，字段级补丁，未知字段/非法值整次拒绝；quotas为accountId→正整数或null，host为主机。默认20/4/2/50；模型必须在现有目录内。公开执行偏好execution_preferences仅返回四个门槛、默认模型与本账号配额，不返回其他账号上限。既有credential_status/set、assistant_key_status/set、poe_key_status/set继续只读布尔状态、只写密钥。
certificate_status → {fingerprint,algorithm:"SHA-256"}；certificate_regenerate → 新指纹并热加载TLS；certificate_installer → {name:"安装证书.cmd",content,fingerprint}，仅公开证书，无私钥。已登录团队用户调用所有settings_*与certificate_*均403；只有execution_preferences进入团队白名单。
session_status与session.updated中的quota为真实任务输出张数{used,limit,unit:"images"}，未配置上限返回null。operation_execute确认签名绑定实际设置及配额；设置或配额变化时旧确认失效。新品接口读取实际数量/费用门槛。


## 4.11 单图统一执行扩展（D036授权）
operation_execute的generate增加可选model和source:text|reference，缺省仍为既有Nano当前参考图行为；模型/分辨率/实际比例必须满足现有目录。text仅允许独立一个generate步骤、SelectionContext必须为空，不得把现有素材当隐含参考；输出数量为1并使用同一费用/配额门槛。后台持久化UUID任务，在收到供应商像素后新增素材，沿用原Poe/Google供应商函数；其余旧接口语义不变。
纯文本撤销将新增素材置于可恢复回收状态，redo恢复；参考图仍新增版本。所有供应商请求先持久submitted，结果先落盘再采用；未知结果不自动重发。asset.updated的完整patch可能代表新素材，projectId确定其所属项目；纯状态patch不创建素材。


## 4.11 本地计划与结果契约（D036授权）
operation_execute增加local_plan:{plan}，plan沿用既有assistant::Plan严格结构（summary/steps/warnings/clarification/output），保留11类本地处理，参数在创建任务前验证。无预览sampleId、无approved标记、无宿主输出路径。output显式设置最终像素/编码；未指定时交付文件默认WebP。图片处理成功但大小目标未达成时保留版本并明确warning，输出descriptor标EXPORT_TARGET_UNREACHABLE，不提供不合格文件。
assistant_context_plan:{projectId,context:SelectionContext,instruction}，返回原Plan与用量；context是唯一素材上下文，团队可用且仍受账号工作区限制。预设读写沿用旧存储接口，前端统一走services适配。
task_list及task事件增加outputs:[{assetId,name,format,bytes,sha256,width,height,quality,status}]，无主机路径；operation_result:{taskId,assetId}返回octet-stream，校验任务归属、文件大小和SHA256后返回。读取结果的ID不作为处理上下文。新增标准处理结果文件在主机临时工作区，客户端恢复文件夹授权后可自动写回。
标准批次undo/redo新增整批状态提交和中断恢复，不删除版本；原命令继续兼容。保存文件的回收/24h清理策略仍需后续P5统一审核，当前不会因新客户端凭据而提前删除主机副本。

## 4.11 自然语言混合管线契约
chat_execute可选pipeline:true、execution:realtime|batch、model，均独立于唯一素材上下文SelectionContext。规划缓存/门槛确认签名包括上述选项；返回统一执行行和任务，不返回宿主路径。内部kind:pipeline承载before/generation/after/output，Batch复用旧引擎并隐藏内部重复任务，保留取消和恢复。
export可选format:png|jpeg|webp、maxBytes；未指定宽高时保留实际尺寸，纯导出任务undoable:false。输出描述仍校验文件SHA和任务工作区。标准、新品及短时规划任务均参与data_root_switch活跃任务保护。

## 4.11 导出页契约补齐（沿用继续工作授权）
operation_execute的export操作增加可选width/height（必须同时出现）、quality、fit、background，保留format/maxBytes；宽高1–16000且总像素不超过1.2亿，quality为1–100整数，fit为pad/crop，background为RGB字节数组，maxBytes为0–1e9整数。未指定尺寸保留实际像素。纯导出仍不新增图片版本、undoable:false、quotaUnits:0，结果使用原有operation_result及无主机路径输出描述。
新增只读selection_export_preview:{projectId,context:SelectionContext,preset:{format,width,height,quality,fit,background,maxBytes},edit,previewIndex?}，保留真实选择范围，previewIndex默认为0且只能定位context目标中的一个素材；不接受独立assetId/ids、outputDir或宿主路径。edit是图像编辑参数数据，不扩展SelectionContext。返回沿用原export_preview的data/before/mime/bytes/quality/width/height/status，无文件写入、模型调用或任务创建；复用原编码器，旧预览/导出命令保留兼容。
客户端保存使用已授权目录句柄；未授权可在结果行保存或下载。旧outputDir偏好保留在本机但不发送或当作已授权。选择范围由target.ts按统一优先级生成，显式选择当前/已选/全部时更新真实选择状态。
