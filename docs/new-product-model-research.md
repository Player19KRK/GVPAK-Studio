# 新品模式：模型与接口核验

核验日期：2026-10-27。范围：Google第一方文档和当前仓库源码；未读取凭据/运行数据，未执行收费模型请求，未修改配置或业务源码。

## 可直接采用的结论

| 用途 | 官方模型ID | 已核实能力与边界 |
| --- | --- | --- |
| 多图识别、参考图选择、结构化任务单 | `gemini-3.8-flash` | 支持图片输入、文本输出和结构化输出；不生成图片；支持Batch；不支持Live API |
| 套图生成 | `gemini-nano-banana-2.1` | 支持图文输入、图像/文本输出；1K/2K/4K，默认1K；最多14张参考图；不支持结构化输出或Live API |

以上两个名称都能在官方模型页确认，不需要猜测或替换为其他模型。仓库`config/models.json`已有Nano Banana 2.1的正确ID，但未配置3.8识别流程。[Gemini 3.8 Flash](https://ai.google.dev/gemini-api/docs/models/gemini-3.8-flash)、[Nano Banana 2.1](https://ai.google.dev/gemini-api/docs/models/gemini-nano-banana-2.1)

识别与生图必须分成两个请求阶段：3.8返回任务JSON，2.1按每个任务的提示词和参考图生成图片。不要要求2.1同时遵守识别任务单的JSON schema。此处“实时”应指普通即时生成请求，并非Google Live API。

## 与现有REST路线兼容

Google当前主文档优先展示Interactions API，但明确表示generateContent仍受支持；官方也保留了3.8的generateContent示例。因此本仓库可沿用reqwest和现有REST方式，不必新增SDK或迁移整套服务。Batch仍只支持generateContent，继续共用该请求结构更便于本轮接入。[迁移说明](https://ai.google.dev/gemini-api/docs/migrate-to-interactions)、[3.8的generateContent说明](https://ai.google.dev/gemini-api/docs/generate-content/whats-new-gemini-3.8)、[Batch说明](https://ai.google.dev/gemini-api/docs/batch-api)

- 识别：`POST /v1beta/models/gemini-3.8-flash:generateContent`。
- 生图：当前仓库使用`POST /v1beta/models/gemini-nano-banana-2.1:generateContent`；官方legacy生图指南还展示了`/v1/models/...:generateContent`。不据此强制改动现有可用版本路径。[官方生图REST示例](https://ai.google.dev/gemini-api/docs/generate-content/image-generation)
- 3.8的thinking支持`low / medium / high`，不支持`minimal`。迁移说明要求移除旧的temperature/top_p/top_k/candidate_count等采样参数。识别初版可选择low作为延迟取舍，但这属于应用选择，不是准确率保证。[3.8参数变化](https://ai.google.dev/gemini-api/docs/generate-content/whats-new-gemini-3.8)

## 多图、标签与输入大小

generateContent允许在同一`contents[].parts`数组放置多张图片，各图片使用`inlineData`或Files API的`fileData`。图片数据使用MIME类型和base64，不应传本机路径。文本标记与图片交错是应用可以采用的组织方式：标记中给出稳定assetId和文件名，紧随对应图像，再要求JSON只返回给定ID。[多图输入指南](https://ai.google.dev/gemini-api/docs/generate-content/image-understanding)、[Content/Part API定义](https://ai.google.dev/api/generate-content)

```json
{
  "contents": [{
    "role": "user",
    "parts": [
      {"text": "识别任务说明与SelectionContext"},
      {"text": "IMAGE assetId=asset-a; fileName=正面.png"},
      {"inlineData": {"mimeType": "image/png", "data": "BASE64_A"}},
      {"text": "IMAGE assetId=asset-b; fileName=侧面.png"},
      {"inlineData": {"mimeType": "image/png", "data": "BASE64_B"}}
    ]
  }]
}
```

图片顺序不是可靠的业务主键；“标记必须跟随对应字节”和“返回ID必须属于输入集合”应由本地代码验证。把文件名/图片内文字视为素材数据，不作为覆盖系统任务的指令。这些属于实现建议。

inline图片的总请求大小应低于20MB（包括提示词、图片和请求内容），更大输入可用Files API。base64会增加体积，应检查序列化后的请求字节数，而不只累加原文件大小；可以先对识别副本缩小尺寸，保留原素材供生图。不要将“可输入很多图片”误当成无限制的一次上传。[图片输入方式与限制](https://ai.google.dev/gemini-api/docs/generate-content/image-understanding)

## 结构化结果与澄清问题

3.8支持JSON Schema的对象、数组、枚举等结构，可定义`questions`数组及每个问题的产品/素材ID、问题文案、选项和受影响任务。官方当前REST示例使用`generationConfig.responseFormat.text={mimeType:"APPLICATION_JSON",schema:{...}}`；SDK示例还可见`responseMimeType`/`responseJsonSchema`，不要把SDK字段说明误当成最新REST首选形状。[结构化输出指南](https://ai.google.dev/gemini-api/docs/generate-content/structured-output)

建议结构（业务设计建议，不是官方固定schema）：

```text
products[]: productId, identity, materialEvidence, visibleTexts, assetIds
assets[]: assetId, productId, role, angle, duplicateOf, confidence
questions[]: id, productId, assetIds, text, options[], affectsJobIds[]
jobs[]: id, productId, kind, referenceAssetIds[], prompt, blockedByQuestionIds[]
```

- `questions`可为空；有不明确的内容物、使用方式或装配关系时按产品合并问题，不对每张图片重复追问。
- 用户示例：空的香体膏替换装纸塑内托，不能把空腔擅自判为应填膏体。问题可为“这些空位需要加入膏体，还是保留空内托？”；仅相关成品/使用/爆炸任务等待答案，其余明确任务可继续准备。
- schema保证格式不能保证事实。仍要检查ID归属、duplicateOf引用、引用图数量、问题与任务关系、未知字段及每个任务的必要参考图。JSON Schema中的uniqueItems等约束并非都由模型端强制执行，应用需自行去重和校验。[结构化输出的校验限制](https://ai.google.dev/gemini-api/docs/structured-output)
- 分清“图中可见”“用户明确提供”“未知”。未知内部结构、内容物、尺寸不得默认为事实；低置信度允许手工选择参考图或回答问题。

## 生图参考图与参数

2.1支持最多14张参考图；官方同时描述最多10个物体、4个人物的一致性能力。这不是“任意14图都能完全还原”的质量保证。套图每个任务宜只选与该画面有关的少量清楚参考图，不把整文件夹无差别传给每张输出。[模型参考图说明](https://ai.google.dev/gemini-api/docs/models/gemini-nano-banana-2.1)

生图请求继续使用多个`inlineData` parts，并用文本说明每张图的角色。`generationConfig.imageConfig`支持`aspectRatio`与`imageSize`；1K/2K/4K大小写按官方示例使用。用户规定的五个基础比例及横竖方向均能在官方输出表中找到。生成模型输出像素落在预设档位，不应把它误称为任意精确尺寸；精确比例/像素要求仍需既有后处理。[生图参数及尺寸表](https://ai.google.dev/gemini-api/docs/generate-content/image-generation)

建议各类参考图角色：白底图以对应角度原图为主；高级场景以完整成品与关键材质/品牌细节为主；模特使用图优先实际成品和可信尺度/握持证据；爆炸图优先拆解/部件图，缺少内部证据时询问或让用户手选。以上是实现取舍，不是官方能力承诺。

## 并行与异步

- 普通即时请求可以由应用使用有限并发池调度。Google的额度按项目计算，受RPM、TPM、RPD及图片模型IPM等限制；没有官方统一“并发N就一定可用”的数值。建议初版默认并发2、可配置小范围，不承诺固定速度。429应按服务提示退避；结果不明的收费请求不自动重新提交。[速率限制](https://ai.google.dev/gemini-api/docs/rate-limits)
- Batch适合非紧急任务，官方目标周转时间为24小时，多数更快；标准费用约一半。异步不等于更快，UI不能把异步模式描述为加速开关。[Batch说明](https://ai.google.dev/gemini-api/docs/batch-api)
- Batch REST：`POST /v1beta/models/{model}:batchGenerateContent`。每项为`{request: GenerateContentRequest, metadata:{key:shotId}}`，所以可为每个shot配置不同提示词/参考图。小批inline请求总量低于20MB；更大任务使用上传JSONL文件。[Batch API定义](https://ai.google.dev/api/batch-api)
- 保存返回的`batches/...`名称，再`GET /v1beta/{name}`轮询；完成后取inline结果或输出文件。终态包括SUCCEEDED、FAILED、CANCELLED、EXPIRED。按metadata.key映射结果，不依赖返回顺序。[Batch任务检查](https://ai.google.dev/gemini-api/docs/batch-api)
- `POST /v1beta/{name}:cancel`取消正在运行的批次，停止继续处理新请求；已完成部分和可能产生的费用不能当作撤回。Batch并发作业有单独限制，不等于实时生图并发额度。[Batch取消](https://ai.google.dev/gemini-api/docs/batch-api#cancelling-a-batch-job)

## 当前仓库的具体接入缺口

以下来自本轮源码阅读，不能当成已实现新品功能：

| 文件 | 当前事实 | 新品模式所需 |
| --- | --- | --- |
| `backend/src/image_provider.rs` | `generate`接收`Option<Vec<u8>>`单张参考；Google请求体只追加一个inlineData | 新增多参考图的独立请求入口/内部方法，保留旧单图调用兼容 |
| `backend/src/main.rs` | `generate`面向一个assetId和单提示词；按目录配置校验model/resolution/ratio | 新品识别任务、任务单、每shot引用ID和并行调度需要独立契约 |
| `backend/src/hybrid.rs` | 一份Recipe/generation prompt应用于多个item；`request`为每item绑定单张prepared图 | 不能直接冒充“每shot独立prompt+多参考图”的任务单 |
| `backend/src/hybrid.rs` | 当前实时tick逐个ready项处理；worker循环按job顺序取任务 | 当前不是新品套图的受限并发池 |
| `backend/src/hybrid.rs` | 已有Batch提交、metadata.key、轮询与结果恢复 | 可复用GoogleREST/结果映射经验，新增功能不应悄悄改变旧任务语义 |
| `backend/src/models.rs` / `config/models.json` | 当前目录包含2.1；配置比率列表没有4:5/5:4，旧Recipe校验也未开放这些比率 | 官方能力已支持；真实接入需要确认后更新相应契约/校验，不能只在前端增加按钮 |

## 验证边界

本次确认的是文档支持和源码形状，没有验证当前用户账号对模型的权限、额度、网络连通性、实际出图质量、参考一致性或费用。最终试用需由用户在界面明确触发已配置的收费分析/生成。没有调用模型列表、识别或生成接口，也没有读取密钥。

## Batch REST最小协议与解析路径（补充核验）

以下为按官方类型定义组合的最小示意，BASE64和任务ID均为占位符，不包含密钥。依据：[Batch REST类型](https://ai.google.dev/api/batch-api)、[官方Batch操作指南](https://ai.google.dev/gemini-api/docs/batch-api)、[生图多参考输入](https://ai.google.dev/gemini-api/docs/generate-content/image-generation)。不要把SDK的`dest`结构直接当成原始REST结构。

### 创建

`POST https://generativelanguage.googleapis.com/v1beta/models/gemini-nano-banana-2.1:batchGenerateContent`

```json
{
  "batch": {
    "displayName": "new-product-job-1-chunk-1",
    "inputConfig": {
      "requests": {
        "requests": [{
          "metadata": {"key": "shot-white-a"},
          "request": {
            "contents": [{
              "role": "user",
              "parts": [
                {"text": "该shot的最终提示词与参考图角色说明"},
                {"text": "REFERENCE assetId=asset-a; role=primary-product"},
                {"inlineData": {"mimeType": "image/png", "data": "BASE64_A"}},
                {"text": "REFERENCE assetId=asset-b; role=material-detail"},
                {"inlineData": {"mimeType": "image/jpeg", "data": "BASE64_B"}}
              ]
            }],
            "generationConfig": {
              "responseModalities": ["TEXT", "IMAGE"],
              "imageConfig": {"aspectRatio": "4:5", "imageSize": "2K"}
            }
          }
        }]
      }
    }
  }
}
```

`requests.requests`的两层不是笔误：`inputConfig.requests`是InlinedRequests对象，它的`requests`字段才是数组。每个shot可以独立拥有prompt、图像parts和参数；整个批次绑定一个model。inline总请求需小于20MB，超过时拆成受限小批或使用文件输入，不能只检查单shot大小。[创建结构](https://ai.google.dev/api/batch-api#InputConfig)

创建结果为Operation，保存其顶层`name`（例`batches/123456`）。不要从displayName猜资源路径；不把提交成功当作图像成功。

### 查询

`GET https://generativelanguage.googleapis.com/v1beta/batches/123456`，无请求正文。

```json
{
  "name": "batches/123456",
  "done": false,
  "metadata": {"state": "JOB_STATE_RUNNING"}
}
```

首先读取顶层`done`和`error`，再读`metadata.state`。官方指南展示`JOB_STATE_*`；API类型页的BatchState枚举使用`BATCH_STATE_*`，两处不一致。实现可按已知后缀PENDING/RUNNING/SUCCEEDED/FAILED/CANCELLED/EXPIRED归一化，未知状态维持等待/诊断，不能当完成。

成功示意中的inline列表存在对象包装：

```json
{
  "name": "batches/123456",
  "done": true,
  "metadata": {"state": "JOB_STATE_SUCCEEDED"},
  "response": {
    "inlinedResponses": {
      "inlinedResponses": [{
        "metadata": {"key": "shot-white-a"},
        "response": {
          "candidates": [{
            "content": {"parts": [
              {"inlineData": {"mimeType": "image/png", "data": "GENERATED_BASE64"}}
            ]}
          }],
          "usageMetadata": {}
        }
      }]
    }
  }
}
```

API类型声明`GenerateContentBatchOutput.inlinedResponses`为InlinedResponses对象，其`inlinedResponses[]`才是列表；官方指南REST示例只把`.response.inlinedResponses`交给jq打印。建议解析两种可观察形状：父字段直接为数组，或父字段对象中的同名数组。当前仓库`hybrid::entries`已经对这些包装作了兼容，可复用该纯解析方法而不改变旧任务逻辑。[响应类型](https://ai.google.dev/api/batch-api#GenerateContentBatchOutput)

逐shot解析：

1. `entry.metadata.key`映射本地shot；不要把不同shot混用。缺失/重复/未知key应明确报错。
2. `entry.error`存在则该shot失败，不解码图片。
3. `entry.response.candidates[].content.parts[]`中选择`thought != true`且含`inlineData.data`的输出，base64解码；保存MIME类型/用量并校验图像字节。
4. 没有图片时检查finishReason/promptFeedback等信息并报告，不能用文本响应冒充出图。
5. 若`response.responsesFile`存在，走`GET https://generativelanguage.googleapis.com/download/v1beta/{responsesFile}:download?alt=media`，逐行解析JSONL中的key/response/error；本轮若只实现inline，可将文件结果标明不支持而不能假定为空成功。

### 取消

`POST https://generativelanguage.googleapis.com/v1beta/batches/123456:cancel`，无请求正文；成功HTTP响应为`{}`。它是异步、尽力取消，随后GET确认；Operation取消错误可能为`error.code=1`，也可能观察到取消终态。响应`{}`本身不代表所有图像都未生成或不会计费。不要用DELETE代替cancel，API参考明确区分二者。[取消方法](https://ai.google.dev/api/batch-api#method:-batches.cancel)
