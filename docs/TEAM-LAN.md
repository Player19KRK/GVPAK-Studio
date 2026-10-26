# GVPAK Studio — small-team LAN mode

## Host

1. Double-click `一键启动.cmd`. The host keeps its existing workspace at http://127.0.0.1:8787/.
2. Open Settings > Team access. Create each colleague account (3–32 ASCII letters/digits/dash/underscore, password at least 12 characters). There is no public registration. Reset re-enables an account and invalidates its existing sessions. Disable immediately denies new API requests and disconnects its event stream within 15 seconds.
3. Once, double-click `开启局域网访问.cmd` and approve Windows administrator elevation. It adds only TCP 8788, Private profile, LocalSubnet, restricted to this application executable. Do not use Public-profile rules or router port forwarding.
4. Send colleagues the HTTPS address shown in Team access and only the certificate `backend/team-private/GVPAK-Team.cer`. Never share server.key or accounts.json.

## Colleagues: one-time certificate trust

Verify the certificate file came directly from the administrator. On Windows double-click GVPAK-Team.cer > Install Certificate > Current User > Place all certificates in the following store > Trusted Root Certification Authorities. Confirm the certificate only after checking with the administrator. Restart the browser, then open the supplied https://HOST-IP:8788 address and sign in. Do not simply bypass a browser certificate warning. Other operating systems require their equivalent trust-store installation.

If the host LAN address changes, restart the service. Its certificate is regenerated when its configured hostnames/IPs change; share and trust the new certificate. A DHCP reservation avoids repeated changes. The host must be awake and connected to the same LAN. Wi-Fi client isolation can prevent access even with correct firewall settings.

## Privacy boundaries

The colleague's browser only uploads explicitly selected/dropped files. It cannot enumerate their disk. Uploaded images are stored on the host inside that account's workspace; this is necessary for host-side processing. Remote accounts cannot browse arbitrary host files, the owner's original projects, or another account's workspace. The physical host administrator still controls the storage and can read files locally.

The local owner service binds exclusively to 127.0.0.1:8787. Account administration and existing host file pickers exist only there. The separate HTTPS service on 8788 enforces login and a strict command allowlist. Path import, data-root switching, native dialogs, shell opening, account management, and credential modification are denied remotely, even with direct API calls. Output paths supplied by clients are overridden with the account's export directory. Downloads are authenticated and resolved only inside that directory. Project IDs and asset IDs are resolved against the account's Store. Events, cancellation and preview supersession are per account.

Passwords use salted PBKDF2-HMAC-SHA256 (600,000 rounds). Session cookies are HttpOnly, Secure, SameSite=Strict, expire after 12 hours, and are invalidated on service restart. Host/origin checks and a per-IP login attempt limit protect authentication. The service has no permissive CORS. API keys stay in the host Windows Credential Manager and are accessed through the same existing profile; colleagues see connection status but cannot view/change keys. AI requests use the host's configured providers and budget; existing approval controls remain.

## Queue and files

Team imports, saves, exports, local processing and direct image generation share a single FIFO processing slot. Background hybrid jobs use the same slot for processing ticks; cloud waiting does not permanently occupy it. Interactive previews and job status remain outside the heavy-task queue. There is no application-level queued-request count cap; one heavy operation runs at a time. Existing hybrid jobs persist; a foreground request waiting in memory must be retried after service restart.

Use the existing Import/Folder controls to pick files on the colleague's computer. Maximum upload request 80 MiB JSON/base64, no total image-count cap, at most 30 MiB per encoded image. The host stages exports in the account's private Downloads area, including per-task subfolders. A remote user can grant a browser-local directory for automatic saving during the current page session; unsupported browsers use standard downloads. Export rows have Download links; the account bar lists all completed exports. The host's native output path chooser is never invoked for remote accounts.

Workspaces, account hashes and TLS private keys are under backend/team-private and excluded from Git and the distribution ZIP. Back them up privately if needed. Never package them for distribution.

## Verification

Run `python scripts/test-team-api.py` after a release build. It starts an isolated test server at 8793/8794, trusts only its generated certificate in the Python test client, creates disposable accounts and images, and verifies isolation. It does not install a certificate in the OS or call paid AI. Rust tests additionally verify queue blocking/release, path rejection, sanitization and command restrictions.

## Updated temporary retention
Team image workspaces now use the policy in [TEMPORARY-WORKSPACES.md](TEMPORARY-WORKSPACES.md): verified local save acknowledgement removes matching payloads; unclaimed images expire after 24 hours. Existing team files receive a 24-hour migration grace. Owner-local projects and feedback remain outside this policy. Browser directory handles can be restored only while their existing browser permission remains granted.
# 按设备保持登录

登录时勾选“在此设备保持登录（30 天）”，当前浏览器可在 30 天内免输密码，主机程序重启后仍有效。每台电脑、每个浏览器分别保存自己的登录凭据，不按 IP 地址识别设备。同一台电脑切换浏览器或清除网站 Cookie 后需要重新登录；无痕浏览关闭后通常不会保留。

不勾选时登录有效期仍为 12 小时，主机重启后需要重新登录。退出登录会撤销当前浏览器的凭据；管理员重置密码或停用账号会使该账号所有设备的旧凭据失效。此功能仅保留登录状态，不在网页存储中保存明文密码。凭据通过 HTTPS 的 HttpOnly、Secure、SameSite=Strict Cookie 保存，主机仅在 `backend/team-private/device-sessions.json` 中持久化凭据哈希和有效期。

