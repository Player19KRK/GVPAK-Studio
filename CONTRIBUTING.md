# 开发与发布维护

## 分支与提交
- 小型维护可按现有方式提交到 main；多人协作或较大改动使用独立分支和 Pull Request。
- 每次提交描述具体变化，例如 `feat: add new product shot workflow`、`fix: restore task output safely`、`docs: update preview release notes`。
- 上传前先同步远端；不要强制推送覆盖他人提交。发布标签必须指向实际上传且通过检查的提交。

## 本地检查
需要 Node.js 24、Rust 2024 edition 工具链和 Python/OpenCV。依赖由 package-lock.json 锁定。

```powershell
npm ci
npm run check
npm test
npm run build
cargo test --manifest-path backend/Cargo.toml
cargo build --release --manifest-path backend/Cargo.toml
```

后端测试需要安装 OpenCV 的 Python，可通过 GVPAK_PYTHON 指定。npm test 不调用云端提供商。Windows CI 执行这些基础检查，不使用真实凭据或用户图片。

scripts/verify-*.mjs、verify-*.py 是额外的浏览器/真实服务验收工具，部分需要独立测试服务、隔离数据和本机生成的夹具，不属于可无条件执行的基础测试。名称包含 cloud、deepseek 的工具可能产生费用；不得在公开 CI 配置密钥或自动运行付费验收。

## 构建与运行
仓库沿用已构建的 dist/ 和 bin/gvpak-studio.exe，以支持 Windows 一键启动。前端构建后只暂存本次构建产生的文件，不要把运行目录里为了旧页面兼容而保留的多轮旧哈希文件全部提交。后端改动须重新构建，并核对 bin 中的程序与 release 构建一致。

更新本机运行版本前保存草稿并检查活跃任务；备份旧入口和程序，核对更新前后的项目 ID、原图哈希、版本数和证书。GitHub 上传不会自动更新其他电脑上的程序。

## 上传前检查
用明确的目录/文件清单暂存改动，再执行：

```powershell
node scripts/check-publication.mjs
git diff --cached --check
git diff --cached --stat
```

检查脚本扫描暂存区的文件名和实际 Git blob，而非仅扫描工作目录。它拒绝运行数据、凭据/证书、日志/测试输出、审阅副本和常见密钥格式，也检查当前 dist 的引用完整性。它不能替代人工审查所有二进制或文档内容。

运行时项目、API 密钥、Windows 凭据、账号、私钥、机器本地设置、.tools/、output/ 和审阅副本始终留在本机。不要以“排除 .env”代替完整检查，也不要上传含这些文件的目录压缩包。

## 更新日志与发布
在 CHANGELOG.md 顶部写明新增、修复、验证和已知限制，README 同步实际交互。未完成必须验收时使用预发布标签（例如 v2.0.0-preview.20261028），GitHub Release 标记为 pre-release，不能写“全部验收通过”。GitHub CI 成功与真实目录/设备验收分别记录。

确认本地与远端提交 SHA 一致后创建标签及 Release。若 CI 失败，修正后追加提交，不改写已发布历史。发布附件只从审核后的 Git 文件或干净构建生成，提供校验和，不包含本机工作目录和运行数据。
